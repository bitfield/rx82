use anyhow::{Context as _, Result, bail};

use crate::{bus::Bus, system::Device};

/// The system RAM.
#[derive(Debug)]
pub struct Memory {
    pub data: Vec<u8>,
    pub end: u16,
    pub start: u16,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            start: 0x0000,
            end: 0xBFFF,
            data: vec![0; 0xC000],
        }
    }
}

impl Device for Memory {
    /// Responds to a memory request if the [`Bus::mem`] line is active.
    fn tick(&mut self, bus: &mut Bus) {
        if bus.mem && self.in_range(bus.addr) {
            match (bus.mem, bus.write) {
                (true, false) => {
                    // Memory read request
                    let data = self.get(bus.addr);
                    bus.write_data(data);
                }
                (true, true) => {
                    // Memory write request
                    self.set(bus.addr, bus.data);
                }
                _ => {}
            }
        }
    }
}

impl Memory {
    /// Returns the byte at logical address `addr`.
    ///
    /// Returns zero if the address is outside the configured memory range.
    #[must_use]
    pub fn get(&self, addr: u16) -> u8 {
        let phys_addr = addr.strict_sub(self.start);
        self.data
            .get(usize::from(phys_addr))
            .copied()
            .unwrap_or_default()
    }

    /// Returns true if `addr` is in the memory's logical address range.
    #[must_use]
    pub fn in_range(&self, addr: u16) -> bool {
        self.start <= addr && addr <= self.end
    }

    /// Loads `data` into memory at address `addr`.
    ///
    /// # Errors
    ///
    /// If the load exceeds bounds.
    pub fn load(&mut self, addr: u16, data: &[u8]) -> Result<()> {
        if !self.in_range(addr) {
            bail!("out of range")
        }
        let phys_addr = addr.saturating_sub(self.start);
        let start = usize::from(phys_addr);
        let end = start.checked_add(data.len()).context("data too long")?;
        let slice = self.data.get_mut(start..end).context("out of bounds")?;
        slice.copy_from_slice(data);
        Ok(())
    }

    /// Sets the byte at address `addr` to `val`.
    ///
    /// If `addr` is out of range, this has no effect.
    pub fn set(&mut self, addr: u16, val: u8) {
        let phys_addr = addr.saturating_sub(self.start);
        if let Some(loc) = self.data.get_mut(usize::from(phys_addr)) {
            *loc = val;
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test")]
mod tests {
    use super::*;

    #[test]
    fn load_checks_bounds_correctly() {
        let mut mem = Memory::default();
        mem.load(0x0000, &[0x00]).expect("valid load failed");
        mem.load(0xBFFF, &[0x00]).expect("valid load failed");
        mem.load(0xBFFE, &[0x00, 0x00, 0x00])
            .expect_err("invalid load succeeded");
        mem.load(0xC000, &[0x00])
            .expect_err("invalid load succeeded");
        mem.load(0xFFFF, &[0x00, 0x00])
            .expect_err("invalid load succeeded");
    }
}
