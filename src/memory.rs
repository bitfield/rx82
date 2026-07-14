use anyhow::{Context as _, Result};

use crate::system::{Bus, BusState, Device};

/// The system memory.
#[non_exhaustive]
#[derive(Debug)]
pub struct Memory(pub Vec<u8>);

impl Default for Memory {
    /// The default [`Memory`] is 64KiB of zeroes.
    #[inline]
    fn default() -> Self {
        Self(vec![0; 0x10_000]) // 64KiB
    }
}

impl Device for Memory {
    /// Responds to a memory request if the [`Bus::mem`] line is active.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        match (bus.mem, bus.write) {
            (true, false) => {
                // Memory read request
                let data = self.get(bus.addr);
                bus.pending_write.get_or_insert(vec![BusState::Data(data)]);
            }
            (true, true) => {
                // Memory write request
                self.set(bus.addr, bus.data);
            }
            _ => {}
        }
    }
}

impl Memory {
    /// Returns the byte at address `addr`.
    ///
    /// Returns zero if the address is outside the configured memory range.
    #[inline]
    #[must_use]
    pub fn get(&self, addr: u16) -> u8 {
        self.0.get(usize::from(addr)).copied().unwrap_or_default()
    }

    /// Loads `data` into memory at address `addr`.
    ///
    /// # Errors
    ///
    /// If the load exceeds bounds.
    #[inline]
    pub fn load(&mut self, addr: u16, data: &[u8]) -> Result<()> {
        let start = usize::from(addr);
        let end = start.checked_add(data.len()).context("data too long")?;
        let slice = self.0.get_mut(start..end).context("out of bounds")?;
        slice.copy_from_slice(data);
        Ok(())
    }

    /// Sets the byte at address `addr` to `val`.
    ///
    /// If `addr` is out of range, this has no effect.
    #[inline]
    pub fn set(&mut self, addr: u16, val: u8) {
        if let Some(loc) = self.0.get_mut(usize::from(addr)) {
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
        mem.load(0xFFFF, &[0x00]).expect("valid load failed");
        mem.load(0xFFFF, &[0x00, 0x00])
            .expect_err("invalid load succeeded");
    }
}
