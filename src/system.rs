use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use crate::{bus::Bus, clock::Clock, cpu::Cpu, device::Device, memory::Memory};

/// The RX82 system as a whole.
#[non_exhaustive]
pub struct System {
    /// The system bus.
    pub bus: Bus,
    /// The system CPU.
    pub cpu: Cpu,
    /// Any attached devices, such as the [`Clock`].
    pub devices: Vec<Box<dyn Device>>,
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
            devices: vec![Box::new(Clock::default())],
            mem: Memory::default(),
            ticks: Default::default(),
        }
    }
}

impl System {
    /// Prints the current cycle count and bus state.
    #[inline]
    pub fn debug_print(&self) {
        println!(
            "Tick {:04X} Addr {:04X} Data {:02X} Mem {}",
            self.ticks, self.bus.addr, self.bus.data, self.bus.mem
        );
    }

    /// Advances the system by one clock cycle.
    ///
    /// # Errors
    ///
    /// If flushing stdout or reading stdin fails in debug mode.
    #[inline]
    pub fn tick(&mut self) -> Result<()> {
        if self.bus.debug {
            let mut input = String::new();
            self.debug_print();
            stdout().flush()?;
            _ = stdin().read_line(&mut input)?;
        }
        self.cpu.tick(&mut self.bus);
        self.mem.tick(&mut self.bus);
        for device in &mut self.devices {
            device.tick(&mut self.bus);
        }
        self.bus.reconcile();
        self.ticks = self.ticks.wrapping_add(1);
        Ok(())
    }
}
