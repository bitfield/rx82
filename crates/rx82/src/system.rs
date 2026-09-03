use anyhow::Result;

use core::fmt::Write as _;

use r8asm::{assemble, disassemble};
use r8cpu::regs::Reg::*;

use crate::{
    bus::Bus,
    clock::Clock,
    cpu::{Cpu, State},
    memory::Memory,
    rom::Rom,
};

/// The RX82 ROM code.
pub const ROM_DATA: &[u8] = include_bytes!("../sys/rx82_rom.bin");

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
pub struct Snapshot {
    /// Bus state at end of tick.
    pub bus: Bus,
    /// CPU state at start of tick.
    pub state: State,
    /// Sequence number of just-completed tick.
    pub tick: u16,
}

/// The RX82 system as a whole.
///
/// Execution proceeds by repeatedly calling [`System::tick`]. On each tick, all devices
/// attached to the system will be ticked by calling their [`Device::tick`] method in
/// turn, in this order:
///
/// 1. CPU
/// 2. Devices in the `devices` list, in order of decreasing priority.
/// 3. Memory
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
    /// Any attached devices, such as the [`Rom`].
    pub devices: Vec<Box<dyn Device>>,
    /// Stored debug snapshots.
    pub history: Vec<Snapshot>,
    /// The system memory.
    pub mem: Memory,
    /// Turbo (max clock speed) mode.
    pub turbo: bool,
}

impl Default for System {
    /// The default `System` has all-default devices.
    fn default() -> Self {
        let mut sys = Self {
            bus: Bus::default(),
            clock: Box::new(Clock::default()),
            cpu: Cpu::default(),
            cycles: 0,
            debug: false,
            devices: Vec::new(),
            history: Vec::new(),
            mem: Memory::default(),
            turbo: false,
        };
        let data = Vec::from(ROM_DATA);
        let rom = Rom {
            start: 0xC000,
            end: 0xFFFF,
            data,
        };
        sys.devices.push(Box::new(rom));
        sys
    }
}

impl System {
    /// Prints the current CPU state and the next instruction in memory.
    pub fn debug_print(&mut self) {
        let next = self.disassemble_next();
        println!("  PC   SP  A  B  C  D  E  F  G  H ZC | NEXT");
        println!(
            "{:04X} {:04X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:1b}{:1b} | {}",
            self.cpu.pc,
            self.cpu.regs.get16(SP),
            self.cpu.regs.get(A),
            self.cpu.regs.get(B),
            self.cpu.regs.get(C),
            self.cpu.regs.get(D),
            self.cpu.regs.get(E),
            self.cpu.regs.get(F),
            self.cpu.regs.get(G),
            self.cpu.regs.get(H),
            u8::from(self.cpu.flags.zero),
            u8::from(self.cpu.flags.carry),
            next,
        );
    }

    /// Returns the disassembly of the instruction at PC.
    #[must_use]
    pub fn disassemble_next(&mut self) -> String {
        let code = vec![
            self.peek_mem(self.cpu.pc),
            self.peek_mem(self.cpu.pc.wrapping_add(1)),
            self.peek_mem(self.cpu.pc.wrapping_add(2)),
        ];
        disassemble(&code)
    }

    /// Reads the contents of system memory at logical address `addr`.
    ///
    /// This may be RAM, ROM, or a memory-mapped I/O device: the monitor puts the
    /// requested address on the bus and ticks the system to service the request, then
    /// reads back the contents of the data bus.
    pub fn peek_mem(&mut self, addr: u16) -> u8 {
        // Save current CPU/bus state
        let halted = self.cpu.halt;
        let bus_state = self.bus.clone();

        // Halt the CPU and set up the bus request
        self.cpu.halt = true;
        self.bus.addr = addr;
        self.bus.mem = true;
        self.bus.write = false;

        // Tick the system once to fulfil the request
        self.tick();

        // Restore the saved state
        self.cpu.halt = halted;
        let data = self.bus.data;
        self.bus = bus_state;
        data
    }

    /// Resets the system (by triggering a CPU reset).
    ///
    /// This resets the CPU to its default state and starts execution from the reset
    /// vector. Memory and other devices are not affected.
    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
        self.run();
    }

    /// Runs the system until halted.
    pub fn run(&mut self) {
        self.cpu.halt = false;
        while !self.cpu.halt {
            self.tick();
        }
    }

    /// Loads `program` at start of user memory and runs until halted.
    ///
    /// # Errors
    ///
    /// If the program does not fit into memory.
    pub fn run_program(&mut self, program: &[u8]) -> Result<()> {
        self.mem.load(0x0100, program)?;
        self.cpu.pc = 0x0100;
        self.run();
        Ok(())
    }

    /// Assembles and runs `program` and prints a trace, panicking on error.
    ///
    /// # Panics
    ///
    /// * On errors from [`assemble_with_debug`] or [`run_program`](Self::run_program).
    #[expect(clippy::unwrap_used, reason = "just for tests")]
    pub fn test_asm(&mut self, source: &str) {
        self.test_prog(&assemble(source).unwrap());
    }

    /// Runs `program` and prints a trace, panicking on error.
    ///
    /// # Panics
    ///
    /// * On errors from [`run_program`](Self::run_program).
    #[expect(clippy::unwrap_used, reason = "just for tests")]
    pub fn test_prog(&mut self, program: &[u8]) {
        self.debug = true;
        self.history = Vec::new();
        self.run_program(program).unwrap();
        self.trace();
    }

    /// Advances the system by one clock cycle.
    pub fn tick(&mut self) {
        let state = self.cpu.state; // save before cpu.tick() overwrites it
        self.cpu.tick(&mut self.bus);
        for device in &mut self.devices {
            device.tick(&mut self.bus);
        }
        self.mem.tick(&mut self.bus);
        self.bus.reconcile();
        if self.debug {
            self.history.push(Snapshot {
                tick: self.cycles,
                state, // at start of this tick
                bus: self.bus.clone(),
            });
        }
        if !self.turbo {
            self.clock.tick(&mut self.bus);
        }
        self.cycles = self.cycles.wrapping_add(1);
    }

    /// Prints a bus timing diagram from the stored history.
    ///
    /// # Panics
    ///
    /// If writing to the strings fails.
    #[expect(clippy::non_ascii_literal, reason = "looks nice")]
    #[expect(clippy::unwrap_used, reason = "panic is okay here")]
    pub fn trace(&self) {
        if self.history.is_empty() {
            return;
        }
        for chunk in self.history.chunks(16) {
            let mut tick = String::from("TICK ");
            let mut header = String::from("─────");
            let mut state = String::from("CPU  ");
            let mut addr = String::from("ADDR ");
            let mut data = String::from("DATA ");
            let mut mem = String::from("/MEM ");
            let mut wrt = String::from("/WRT ");
            for snapshot in chunk {
                write!(tick, " {:04X}", snapshot.tick).unwrap();
                write!(header, "─────").unwrap();
                write!(state, " {}", snapshot.state).unwrap();
                write!(addr, " {:04X}", snapshot.bus.addr).unwrap();
                write!(data, " ──{:02X}", snapshot.bus.data).unwrap();
                write!(
                    mem,
                    "{}",
                    if snapshot.bus.mem {
                        " ─MEM"
                    } else {
                        " ────"
                    }
                )
                .unwrap();
                write!(
                    wrt,
                    "{}",
                    if snapshot.bus.write {
                        " ─WRT"
                    } else {
                        " ────"
                    }
                )
                .unwrap();
            }
            println!("{tick}");
            println!("{header}");
            println!("{state}");
            println!("{addr}");
            println!("{data}");
            println!("{mem}");
            println!("{wrt}");
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r8cpu::instructions::InstructionKind::*;

    #[test]
    fn trace_formatting_copes_with_long_lines() {
        let mut sys = System::default();
        let mut nops = vec![u8::from(Nop); 7];
        nops.push(u8::from(Halt));
        sys.test_prog(&nops);
        // panic!("uncomment me to check trace formatting");
    }
}
