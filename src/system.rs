use anyhow::{Result, ensure};

use core::fmt::Write as _;

use crate::{
    asm::disassemble,
    clock::Clock,
    cpu::{Cpu, Phase},
    memory::Memory,
    regs::Reg8::{A, B},
};

/// The system bus.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
#[expect(clippy::partial_pub_fields, reason = "pending_write is internal")]
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
    pending_write: Option<Vec<State>>,
}

impl Bus {
    /// Asserts all the given bus states.
    ///
    /// # Errors
    ///
    /// On the first failed assertion.
    #[inline]
    pub fn assert(&self, states: &[State], msg: &'static str) -> Result<()> {
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
            }
        }
        Ok(())
    }

    /// Tries to set `states` on the bus at the end of this cycle.
    ///
    /// If a write is already pending, this has no effect.
    #[inline]
    pub fn defer_write(&mut self, states: Vec<State>) {
        if self.pending_write.is_none() {
            self.pending_write = Some(states);
        }
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
                }
            }
        }
    }
}

/// The trait that all devices connected to the [`Bus`] implement.
pub trait Device {
    /// Notifies the device that a new clock cycle has begun.
    fn tick(&mut self, bus: &mut Bus);
}

/// A snapshot of the system state for debugging.
#[non_exhaustive]
pub struct Snapshot {
    pub bus: Bus,
    pub phase: Phase,
    pub tick: u16,
}

/// A desired or asserted bus state.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum State {
    Addr(u16),
    Data(u8),
    Mem(bool),
}

/// The RX82 system as a whole.
#[non_exhaustive]
pub struct System {
    /// The system bus.
    pub bus: Bus,
    /// The system CPU.
    pub cpu: Cpu,
    /// Enable debug snapshots.
    pub debug: bool,
    /// Any attached devices, such as the [`Clock`].
    pub devices: Vec<Box<dyn Device>>,
    /// Stored debug snapshots.
    pub history: Vec<Snapshot>,
    /// The system memory.
    pub mem: Memory,
    /// Cycle counter.
    pub ticks: u16,
}

impl Default for System {
    /// The default `System` has all-default devices, including a default [`Clock`].
    #[inline]
    fn default() -> Self {
        Self {
            bus: Bus::default(),
            cpu: Cpu::default(),
            debug: false,
            devices: vec![Box::new(Clock::default())],
            history: Vec::new(),
            mem: Memory::default(),
            ticks: Default::default(),
        }
    }
}

impl System {
    /// Prints the current CPU state and the next instruction in memory.
    #[inline]
    pub fn debug_cpu(&self) {
        println!("  PC  A  B NEXT");
        println!(
            "{:04X} {:02X} {:02X} {}",
            self.cpu.pc,
            self.cpu.regs.get8(A),
            self.cpu.regs.get8(B),
            disassemble(self.mem.slice_from(self.cpu.pc))
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
                tick: self.ticks,
                phase, // at start of this tick
                bus: self.bus.clone(),
            });
        }
        self.ticks = self.ticks.wrapping_add(1);
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
        let mut tick = String::from("TICK ");
        let mut header = String::from("─────");
        let mut phase = String::from("CPU  ");
        let mut addr = String::from("ADDR ");
        let mut data = String::from("DATA ");
        let mut mem = String::from("/MEM ");
        for snapshot in &self.history {
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
    }
}
