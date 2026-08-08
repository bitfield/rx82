use crate::{bus::Bus, system::Device};

/// A system ROM.
#[derive(Debug)]
pub struct Rom {
    /// Holds the ROM data.
    pub data: Vec<u8>,
    /// The last logical address mapped to this ROM.
    pub end: u16,
    /// The first logical address mapped to this ROM.
    pub start: u16,
}

impl Device for Rom {
    /// Responds to a memory request if the [`Bus::mem`] line is active.
    fn tick(&mut self, bus: &mut Bus) {
        if bus.mem && self.in_range(bus.addr) {
            let data = self.get(bus.addr);
            bus.write_data(data);
        }
    }
}

impl Rom {
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

    /// Returns true if logical address `addr` is in the ROM's address range.
    #[must_use]
    pub fn in_range(&self, addr: u16) -> bool {
        self.start <= addr && addr <= self.end
    }
}

#[cfg(test)]
mod tests {
    use crate::{asm::assemble, regs::Reg::A, system::System};

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
        sys.trace_program(&assemble(
            "
            ld cd, 0xBFFF
            ld a, (cd)
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0x00, "wrong A");
        sys.trace_program(&assemble(
            "
            ld cd, 0xB000
            ld a, (cd)
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0xF0, "wrong A");
        sys.trace_program(&assemble(
            "
            ld cd, 0xB001
            ld a, (cd)
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0xF1, "wrong A");
        sys.trace_program(&assemble(
            "
            ld cd, 0xB002
            ld a, (cd)
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0xF2, "wrong A");
    }
}
