use crate::{bus::Bus, device::Device};

#[non_exhaustive]
#[derive(Debug)]
pub struct Memory(Vec<u8>);

impl Default for Memory {
    #[inline]
    fn default() -> Self {
        Self(vec![0; 0xFFFF]) // 64KiB
    }
}

impl Device for Memory {
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        if bus.mem && !bus.dirty {
            bus.data = self.get(bus.addr);
            bus.dirty = true;
        }
    }
}

impl Memory {
    #[inline]
    #[must_use]
    pub fn get(&self, addr: u16) -> u8 {
        self.0.get(usize::from(addr)).copied().unwrap_or_default()
    }

    #[inline]
    pub fn set(&mut self, addr: u16, val: u8) {
        if let Some(loc) = self.0.get_mut(usize::from(addr)) {
            *loc = val;
        }
    }
}
