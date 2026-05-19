use rand::RngExt as _;

#[expect(clippy::min_ident_chars, reason = "single-letter register names")]
#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum Reg8 {
    A,
}

#[derive(Debug)]
pub struct Regs {
    ra: u8,
}

impl Default for Regs {
    #[inline]
    fn default() -> Self {
        let mut rng = rand::rng();
        Self { ra: rng.random() }
    }
}

impl Regs {
    #[inline]
    #[must_use]
    pub fn get8(&self, reg: &Reg8) -> u8 {
        match reg {
            &Reg8::A => self.ra,
        }
    }

    #[inline]
    pub fn set8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.ra = val,
        }
    }
}
