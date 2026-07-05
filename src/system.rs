use anyhow::{Result, ensure};

use core::fmt::Write as _;

use crate::{
    asm::disassemble,
    clock::Clock,
    cpu::{Cpu, Phase},
    memory::Memory,
    regs::Reg::*,
};

/// The system bus.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct Bus {
    /// The 16-bit address bus.
    pub addr: u16,
    /// The 8-bit data bus.
    pub data: u8,
    /// Enables verbose debugging.
    pub debug: bool,
    /// CPU 'memory request' line.
    pub mem: bool,
    /// A possible pending write to the bus state during the current cycle.
    pub pending_write: Option<Vec<State>>,
    /// CPU 'write request' line.
    pub write: bool,
}

impl Bus {
    /// Asserts all the given bus states.
    ///
    /// # Errors
    ///
    /// On the first failed assertion.
    #[inline]
    pub fn assert(&self, states: &[State], msg: impl AsRef<str>) -> Result<()> {
        let msg = msg.as_ref();
        for state in states {
            match *state {
                State::Addr(addr) => ensure!(
                    self.addr == addr,
                    "want bus addr {:04X}, got {:04X} {msg}",
                    addr,
                    self.addr
                ),
                State::Data(data) => ensure!(
                    self.data == data,
                    "want bus data {:02X}, got {:02X} {msg}",
                    data,
                    self.data
                ),
                State::Mem(mem) => ensure!(
                    self.mem == mem,
                    "/MEM line {} {msg}",
                    if self.mem { "active" } else { "inactive" }
                ),
                State::Write(wr) => ensure!(
                    self.write == wr,
                    "/WR line {} {msg}",
                    if self.write { "active" } else { "inactive" }
                ),
            }
        }
        Ok(())
    }

    /// Applies any pending write to the bus.
    #[inline]
    pub fn reconcile(&mut self) {
        if let Some(states) = self.pending_write.take() {
            for state in states {
                match state {
                    State::Addr(addr) => self.addr = addr,
                    State::Data(data) => self.data = data,
                    State::Mem(mem) => self.mem = mem,
                    State::Write(wr) => self.write = wr,
                }
            }
        }
    }
}

/// The trait that all devices connected to the [`Bus`] implement.
pub trait Device {
    /// Notifies the device that a new clock cycle has begun.
    ///
    /// The implementation of `tick` should be fast (<<250ns), because there may be any
    /// number of devices connected to the system, each of which executes one tick per
    /// system clock cycle. There is no timeout, so a slow device will affect the
    /// system's achieved cycle rate.
    fn tick(&mut self, bus: &mut Bus);
}

/// A snapshot of the system state for debugging.
#[non_exhaustive]
pub struct Snapshot {
    /// Bus state at end of tick.
    pub bus: Bus,
    /// CPU phase at start of tick.
    pub phase: Phase,
    /// Sequence number of just-completed tick.
    pub tick: u16,
}

/// A desired or asserted bus state.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum State {
    /// Address bus value.
    Addr(u16),
    /// Data bus value.
    Data(u8),
    /// `/MEM` line state.
    Mem(bool),
    /// `/WR` line state.
    Write(bool),
}

/// The RX82 system as a whole.
///
/// Execution proceeds by repeatedly calling [`System::tick`]. On each tick, all devices
/// attached to the system will be ticked by calling their [`Device::tick`] method in
/// turn, in this order:
///
/// 1. CPU
/// 2. Memory
/// 3. Devices in the `devices` list, in order of decreasing priority.
/// 4. Clock
///
/// Each device has a chance to write to the bus on its tick, but if a higher-priority
/// device has already written to the bus, this attempted write will be ignored. This
/// models a simple daisy-chained “bus grant” arbitration scheme.
///
/// The clock is ticked last, after everything else has been done, because its job is
/// solely to regulate the system cycle rate by waiting (if necessary) until the next
/// cycle is actually due. Thus the clock can only slow down a speeding system, not
/// speed up a slow one.
#[non_exhaustive]
pub struct System {
    /// The system bus.
    pub bus: Bus,
    /// The system clock.
    pub clock: Box<dyn Device>,
    /// The system CPU.
    pub cpu: Cpu,
    /// Cycle counter.
    pub cycles: u16,
    /// Enable debug snapshots.
    pub debug: bool,
    /// Any attached devices, such as the [`Clock`].
    pub devices: Vec<Box<dyn Device>>,
    /// Stored debug snapshots.
    pub history: Vec<Snapshot>,
    /// The system memory.
    pub mem: Memory,
}

impl Default for System {
    /// The default `System` has all-default devices.
    #[inline]
    fn default() -> Self {
        Self {
            bus: Bus::default(),
            clock: Box::new(Clock::default()),
            cpu: Cpu::default(),
            debug: false,
            devices: Vec::new(),
            history: Vec::new(),
            mem: Memory::default(),
            cycles: 0,
        }
    }
}

impl System {
    /// Prints the current CPU state and the next instruction in memory.
    #[inline]
    pub fn debug_print(&self) {
        println!("  PC  A  B  C  D  E  F  G  H  Z | NEXT");
        println!(
            "{:04X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}  {:1b} | {}",
            self.cpu.pc,
            self.cpu.regs.get8(A),
            self.cpu.regs.get8(B),
            self.cpu.regs.get8(C),
            self.cpu.regs.get8(D),
            self.cpu.regs.get8(E),
            self.cpu.regs.get8(F),
            self.cpu.regs.get8(G),
            self.cpu.regs.get8(H),
            u8::from(self.cpu.flags.zero),
            disassemble(
                self.mem
                    .0
                    .get(usize::from(self.cpu.pc)..)
                    .unwrap_or_default()
            )
            .unwrap_or_default(),
        );
    }

    /// Runs the system until halted.
    #[inline]
    pub fn run(&mut self) {
        self.cpu.halt = false;
        while !self.cpu.halt {
            self.tick();
        }
    }

    /// Loads `program` at start of memory and runs until halted.
    ///
    /// # Errors
    ///
    /// If the program does not fit into memory.
    #[inline]
    pub fn run_program(&mut self, program: &[u8]) -> Result<()> {
        self.mem.load(0x0000, program)?;
        self.cpu.pc = 0x0000;
        self.run();
        Ok(())
    }

    /// Advances the system by one clock cycle.
    #[inline]
    pub fn tick(&mut self) {
        let phase = self.cpu.phase; // save before cpu.tick() overwrites it
        self.cpu.tick(&mut self.bus);
        self.mem.tick(&mut self.bus);
        for device in &mut self.devices {
            device.tick(&mut self.bus);
        }
        self.bus.reconcile();
        if self.debug {
            self.history.push(Snapshot {
                tick: self.cycles,
                phase, // at start of this tick
                bus: self.bus.clone(),
            });
        }
        self.clock.tick(&mut self.bus);
        self.cycles = self.cycles.wrapping_add(1);
    }

    /// Prints a bus timing diagram from the stored history.
    ///
    /// # Panics
    ///
    /// If writing to the strings fails.
    #[expect(clippy::non_ascii_literal, reason = "looks nice")]
    #[expect(clippy::unwrap_used, reason = "panic is okay here")]
    #[inline]
    pub fn trace(&self) {
        if self.history.is_empty() {
            return;
        }
        for chunk in self.history.chunks(16) {
            let mut tick = String::from("TICK ");
            let mut header = String::from("─────");
            let mut phase = String::from("CPU  ");
            let mut addr = String::from("ADDR ");
            let mut data = String::from("DATA ");
            let mut mem = String::from("/MEM ");
            for snapshot in chunk {
                write!(tick, " {:04X}", snapshot.tick).unwrap();
                write!(header, "─────").unwrap();
                write!(phase, " {}", snapshot.phase).unwrap();
                write!(addr, " {:04X}", snapshot.bus.addr).unwrap();
                write!(data, " ──{:02X}", snapshot.bus.data).unwrap();
                write!(
                    mem,
                    "{}",
                    if snapshot.bus.mem {
                        " ████"
                    } else {
                        " ────"
                    }
                )
                .unwrap();
            }
            println!("{tick}");
            println!("{header}");
            println!("{phase}");
            println!("{addr}");
            println!("{data}");
            println!("{mem}");
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::InstructionKind::*;

    #[expect(clippy::unwrap_used, reason = "test")]
    #[test]
    fn trace_formatting_copes_with_long_lines() {
        let mut sys = System {
            debug: true,
            ..System::default()
        };
        let mut nops = vec![u8::from(Nop); 7];
        nops.push(u8::from(Halt));
        sys.run_program(&nops).unwrap();
        sys.trace();
        // panic!("uncomment me to check trace formatting");
    }
}
