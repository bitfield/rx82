#[expect(
    clippy::min_ident_chars,
    reason = "R8 uses single-letter register names"
)]
#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum Reg8 {
    A,
    B,
}

#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum Reg16 {
    AB,
}

/// The CPU registers.
#[derive(Debug, Default)]
pub struct Regs {
    ra: u8,
    rb: u8,
}

impl Regs {
    /// Returns the byte in register `reg`.
    #[inline]
    #[must_use]
    pub fn get16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AB => u16::from_le_bytes([self.ra, self.rb]),
        }
    }

    /// Returns the byte in register `reg`.
    #[inline]
    #[must_use]
    pub fn get8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.ra,
            Reg8::B => self.rb,
        }
    }

    /// Sets register pair `reg` to the word `val`.
    #[inline]
    pub fn set16(&mut self, reg: Reg16, val: u16) {
        match reg {
            Reg16::AB => [self.ra, self.rb] = val.to_le_bytes(),
        }
    }

    /// Sets register `reg` to the byte `val`.
    #[inline]
    pub fn set8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.ra = val,
            Reg8::B => self.rb = val,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::regs::{Reg8::*, Reg16::AB};

    use super::*;

    #[test]
    fn set8_and_get8_regs_works() {
        let mut regs = Regs::default();
        regs.set8(A, 0xFF);
        assert_eq!(regs.get8(A), 0xFF, "wrong A");
        regs.set8(B, 0xFF);
        assert_eq!(regs.get8(B), 0xFF, "wrong B");
    }

    #[test]
    fn set16_and_get16_regs_works() {
        let mut regs = Regs::default();
        regs.set16(AB, 0xBEEF);
        assert_eq!(regs.get16(AB), 0xBEEF, "wrong AB");
    }
}
