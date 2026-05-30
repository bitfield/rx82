#[expect(
    clippy::min_ident_chars,
    reason = "R8 uses single-letter register names"
)]
#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum Reg8 {
    A,
}

/// The CPU registers.
#[derive(Debug, Default)]
pub struct Regs {
    /// Accumulator 1.
    ra: u8,
}

impl Regs {
    /// Returns the byte in register `reg`.
    #[inline]
    #[must_use]
    pub fn get8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.ra,
        }
    }

    /// Sets register `reg` to the byte `val`.
    #[inline]
    pub fn set8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.ra = val,
        }
    }
}
