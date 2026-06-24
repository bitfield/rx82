use anyhow::bail;

use core::{
    fmt::{Display, Formatter},
    str::FromStr,
};

/// The 8-bit registers.
#[expect(
    clippy::min_ident_chars,
    reason = "R8 uses single-letter register names"
)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    Invalid(u8),
}

impl Display for Reg8 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Reg8::A => "a",
                Reg8::B => "b",
                Reg8::C => "c",
                Reg8::D => "d",
                Reg8::E => "e",
                Reg8::F => "f",
                Reg8::G => "g",
                Reg8::H => "h",
                Reg8::Invalid(_) => "???",
            }
        )
    }
}

impl FromStr for Reg8 {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(value: &str) -> Result<Self, anyhow::Error> {
        match value {
            "a" => Ok(Reg8::A),
            "b" => Ok(Reg8::B),
            "c" => Ok(Reg8::C),
            "d" => Ok(Reg8::D),
            "e" => Ok(Reg8::E),
            "f" => Ok(Reg8::F),
            "g" => Ok(Reg8::G),
            "h" => Ok(Reg8::H),
            reg => bail!("invalid register {reg}"),
        }
    }
}

impl From<Reg8> for u8 {
    #[inline]
    fn from(reg: Reg8) -> Self {
        match reg {
            Reg8::A => 0x00,
            Reg8::B => 0x01,
            Reg8::C => 0x02,
            Reg8::D => 0x03,
            Reg8::E => 0x04,
            Reg8::F => 0x05,
            Reg8::G => 0x06,
            Reg8::H => 0x07,
            Reg8::Invalid(_) => unreachable!("tried to encode invalid register name"),
        }
    }
}

impl From<u8> for Reg8 {
    #[inline]
    fn from(id: u8) -> Self {
        match id {
            0x00 => Reg8::A,
            0x01 => Reg8::B,
            0x02 => Reg8::C,
            0x03 => Reg8::D,
            0x04 => Reg8::E,
            0x05 => Reg8::F,
            0x06 => Reg8::G,
            0x07 => Reg8::H,
            _ => Reg8::Invalid(id),
        }
    }
}

/// The 16-bit register pairs.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg16 {
    AB,
    CD,
    EF,
    GH,
    Invalid(u8),
}

impl Display for Reg16 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Reg16::AB => "ab",
                Reg16::CD => "cd",
                Reg16::EF => "ef",
                Reg16::GH => "gh",
                Reg16::Invalid(_) => "???",
            }
        )
    }
}

impl FromStr for Reg16 {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(value: &str) -> Result<Self, anyhow::Error> {
        match value {
            "ab" => Ok(Reg16::AB),
            "cd" => Ok(Reg16::CD),
            "ef" => Ok(Reg16::EF),
            "gh" => Ok(Reg16::GH),
            reg => bail!("invalid register {reg}"),
        }
    }
}

impl From<Reg16> for u8 {
    #[inline]
    fn from(reg: Reg16) -> Self {
        match reg {
            Reg16::AB => 0x08,
            Reg16::CD => 0x09,
            Reg16::EF => 0x0A,
            Reg16::GH => 0x0B,
            Reg16::Invalid(_) => unreachable!("tried to encode invalid register name"),
        }
    }
}

impl From<u8> for Reg16 {
    #[inline]
    fn from(id: u8) -> Self {
        match id {
            0x08 => Reg16::AB,
            0x09 => Reg16::CD,
            0x0A => Reg16::EF,
            0x0B => Reg16::GH,
            _ => Reg16::Invalid(id),
        }
    }
}

/// The CPU registers.
#[derive(Debug, Default)]
pub struct Regs {
    ra: u8,
    rb: u8,
    rc: u8,
    rd: u8,
    re: u8,
    rf: u8,
    rg: u8,
    rh: u8,
}

impl Regs {
    /// Returns the word in register pair `reg`.
    #[inline]
    #[must_use]
    pub fn get16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::AB => u16::from_be_bytes([self.ra, self.rb]),
            Reg16::CD => u16::from_be_bytes([self.rc, self.rd]),
            Reg16::EF => u16::from_be_bytes([self.re, self.rf]),
            Reg16::GH => u16::from_be_bytes([self.rg, self.rh]),
            Reg16::Invalid(_) => unreachable!("tried to read invalid register"),
        }
    }

    /// Returns the byte in register `reg`.
    #[inline]
    #[must_use]
    pub fn get8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.ra,
            Reg8::B => self.rb,
            Reg8::C => self.rc,
            Reg8::D => self.rd,
            Reg8::E => self.re,
            Reg8::F => self.rf,
            Reg8::G => self.rg,
            Reg8::H => self.rh,
            Reg8::Invalid(_) => unreachable!("tried to read invalid register"),
        }
    }

    /// Sets register pair `reg` to the word `val`.
    #[inline]
    pub fn set16(&mut self, reg: Reg16, val: u16) {
        match reg {
            Reg16::AB => [self.ra, self.rb] = val.to_be_bytes(),
            Reg16::CD => [self.rc, self.rd] = val.to_be_bytes(),
            Reg16::EF => [self.re, self.rf] = val.to_be_bytes(),
            Reg16::GH => [self.rg, self.rh] = val.to_be_bytes(),
            Reg16::Invalid(_) => unreachable!("tried to set invalid register"),
        }
    }

    /// Sets register `reg` to the byte `val`.
    #[inline]
    pub fn set8(&mut self, reg: Reg8, val: u8) {
        match reg {
            Reg8::A => self.ra = val,
            Reg8::B => self.rb = val,
            Reg8::C => self.rc = val,
            Reg8::D => self.rd = val,
            Reg8::E => self.re = val,
            Reg8::F => self.rf = val,
            Reg8::G => self.rg = val,
            Reg8::H => self.rh = val,
            Reg8::Invalid(_) => unreachable!("tried to set invalid register"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::regs::{Reg8::*, Reg16::AB};

    use super::*;

    #[test]
    fn addressing_individual_regs_works() {
        let mut regs = Regs::default();
        regs.set8(A, 0xFF);
        assert_eq!(regs.get8(A), 0xFF, "wrong A");
        regs.set8(B, 0xFF);
        assert_eq!(regs.get8(B), 0xFF, "wrong B");
    }

    #[test]
    fn addressing_reg_pairs_works() {
        let mut regs = Regs::default();
        regs.set8(A, 0xDE);
        regs.set8(B, 0xAD);
        assert_eq!(regs.get16(AB), 0xDEAD, "wrong AB");
        regs.set16(AB, 0xBEEF);
        assert_eq!(regs.get16(AB), 0xBEEF, "wrong AB");
        assert_eq!(regs.get8(A), 0xBE, "wrong A");
        assert_eq!(regs.get8(B), 0xEF, "wrong B");
    }
}
