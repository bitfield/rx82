use anyhow::{Context as _, Result};

use crate::{
    bus::{Bus, State},
    device::Device,
};

#[non_exhaustive]
#[derive(Debug)]
pub struct Memory(Vec<u8>);

impl Default for Memory {
    #[inline]
    fn default() -> Self {
        Self(vec![0; 0x10_000]) // 64KiB
    }
}

impl Device for Memory {
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        if bus.mem {
            let data = self.get(bus.addr);
            bus.defer_write(vec![State::Data(data)]);
        }
    }
}

impl Memory {
    #[inline]
    #[must_use]
    pub fn get(&self, addr: u16) -> u8 {
        self.0.get(usize::from(addr)).copied().unwrap_or_default()
    }

    /// Load `data` into memory at address `addr`.
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
