use core::fmt::Write as _;

use crate::{
    asm::disassemble,
    bus::Bus,
    clock::Clock,
    cpu::{Cpu, Phase},
    device::Device,
    memory::Memory,
    regs::Reg8::A,
};

/// A snapshot of the system state for debugging.
#[non_exhaustive]
pub struct Snapshot {
    pub bus: Bus,
    pub phase: Phase,
    pub tick: u16,
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
    /// Prints the current cycle count and bus state.
    #[inline]
    pub fn debug_cpu(&self) {
        println!("  PC  A NEXT");
        println!(
            "{:04X} {:02X} {}",
            self.cpu.pc,
            self.cpu.regs.get8(A),
            disassemble(self.mem.slice_from(self.cpu.pc))
        );
    }

    /// Runs the system until a `/HLT` signal is raised.
    #[inline]
    pub fn run(&mut self) {
        self.cpu.halt = false;
        self.bus.halt = false;
        while !self.bus.halt {
            self.tick();
        }
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

    /// Prints a timing diagram from the stored history.
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
        let mut halt = String::from("/HLT ");
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
            write!(
                halt,
                "{}",
                if snapshot.bus.halt {
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
        println!("{halt}");
    }
}
