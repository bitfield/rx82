use crate::{bus::Bus, system::Device};

/// A ROM memory.
#[non_exhaustive]
#[derive(Debug)]
pub struct Rom {
    pub data: Vec<u8>,
    pub end: u16,
    pub start: u16,
}

impl Device for Rom {
    /// Responds to a memory request if the [`Bus::mem`] line is active.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        if bus.mem && self.in_range(bus.addr) {
            let data = self.get(bus.addr);
            bus.write_data(data);
        }
    }
}

impl Rom {
    /// Returns the byte at address `addr`.
    ///
    /// Returns zero if the address is outside the configured memory range.
    #[inline]
    #[must_use]
    pub fn get(&self, log_addr: u16) -> u8 {
        let addr = log_addr.strict_sub(self.start);
        self.data
            .get(usize::from(addr))
            .copied()
            .unwrap_or_default()
    }

    /// Returns true if `addr` is in the ROM's address range.
    #[inline]
    #[must_use]
    pub fn in_range(&self, addr: u16) -> bool {
        self.start <= addr && addr <= self.end
    }
}

#[cfg(test)]
mod tests {
    use crate::{asm::asm, regs::Reg::A, system::System};

    use super::*;

    #[expect(clippy::unwrap_used, reason = "test")]
    #[test]
    fn rom_responds_to_in_range_requests() {
        let mut sys = System::default();
        sys.devices.push(Box::new(Rom {
            start: 0xB000,
            end: 0xB002,
            data: vec![0xF0, 0xF1, 0xF2],
        }));
        sys.run_program(&asm("
            ld cd, 0xBFFF
            ld a, (cd)
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0x00, "wrong A");
        sys.run_program(&asm("
            ld cd, 0xB000
            ld a, (cd)
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0xF0, "wrong A");
        sys.run_program(&asm("
            ld cd, 0xB001
            ld a, (cd)
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0xF1, "wrong A");
        sys.run_program(&asm("
            ld cd, 0xB002
            ld a, (cd)
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0xF2, "wrong A");
    }
}
