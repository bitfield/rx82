use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use crate::{bus::Bus, cpu::Cpu, device::Device as _, memory::Memory};

#[non_exhaustive]
#[derive(Debug, Default)]
pub struct System {
    pub bus: Bus,
    pub cpu: Cpu,
    pub mem: Memory,
    pub ticks: u16,
}

impl System {
    /// Print the current system state.
    ///
    /// # Errors
    ///
    /// If flushing stdout or reading stdin fails.
    #[expect(clippy::use_debug, reason = "temporary")]
    #[inline]
    pub fn debug_print(&self) -> Result<()> {
        let mut input = String::new();
        println!(
            "Tick {:04X} Phase: {:?} Addr {:04X} Data {:02X} Mem {}",
            self.ticks, self.cpu.phase, self.bus.addr, self.bus.data, self.bus.mem
        );
        stdout().flush()?;
        _ = stdin().read_line(&mut input)?;
        Ok(())
    }

    #[inline]
    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.bus);
        self.mem.tick(&mut self.bus);
        self.bus.reconcile();
        self.ticks = self.ticks.wrapping_add(1);
    }
}
