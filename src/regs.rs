use anyhow::{anyhow, bail};

use core::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use crate::cpu::Flags;

/// The 8-bit registers.
#[expect(clippy::min_ident_chars, reason = "the actual names")]
#[expect(clippy::arbitrary_source_item_ordering, reason = "logical order")]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    AB,
    CD,
    EF,
    GH,
}

impl Display for Reg {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Reg::A => "a",
                Reg::B => "b",
                Reg::C => "c",
                Reg::D => "d",
                Reg::E => "e",
                Reg::F => "f",
                Reg::G => "g",
                Reg::H => "h",
                Reg::AB => "ab",
                Reg::CD => "cd",
                Reg::EF => "ef",
                Reg::GH => "gh",
            }
        )
    }
}

impl FromStr for Reg {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(value: &str) -> Result<Self, anyhow::Error> {
        match value {
            "a" => Ok(Reg::A),
            "b" => Ok(Reg::B),
            "c" => Ok(Reg::C),
            "d" => Ok(Reg::D),
            "e" => Ok(Reg::E),
            "f" => Ok(Reg::F),
            "g" => Ok(Reg::G),
            "h" => Ok(Reg::H),
            "ab" => Ok(Reg::AB),
            "cd" => Ok(Reg::CD),
            "ef" => Ok(Reg::EF),
            "gh" => Ok(Reg::GH),
            reg => bail!("invalid register {reg}"),
        }
    }
}

impl From<Reg> for u8 {
    #[inline]
    fn from(reg: Reg) -> Self {
        match reg {
            Reg::A => 0x00,
            Reg::B => 0x01,
            Reg::C => 0x02,
            Reg::D => 0x03,
            Reg::E => 0x04,
            Reg::F => 0x05,
            Reg::G => 0x06,
            Reg::H => 0x07,
            Reg::AB => 0x08,
            Reg::CD => 0x09,
            Reg::EF => 0x0A,
            Reg::GH => 0x0B,
        }
    }
}

impl TryFrom<u8> for Reg {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            0x00 => Ok(Reg::A),
            0x01 => Ok(Reg::B),
            0x02 => Ok(Reg::C),
            0x03 => Ok(Reg::D),
            0x04 => Ok(Reg::E),
            0x05 => Ok(Reg::F),
            0x06 => Ok(Reg::G),
            0x07 => Ok(Reg::H),
            0x08 => Ok(Reg::AB),
            0x09 => Ok(Reg::CD),
            0x0A => Ok(Reg::EF),
            0x0B => Ok(Reg::GH),
            _ => Err(anyhow!("invalid register id {id:#04X}")),
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
    /// Compares the value in register `reg` with the operand, updating `flags`.
    #[inline]
    pub fn cmp(&mut self, reg: Reg, hi: u8, lo: u8, flags: &mut Flags) {
        use crate::regs::Reg::*;
        let (lhs, rhs) = match reg {
            A | B | C | D | E | F | G | H => (u16::from(self.get8(reg)), u16::from(lo)),
            AB | CD | EF | GH => (self.get16(reg), u16::from_be_bytes([hi, lo])),
        };
        flags.zero = lhs == rhs;
        flags.carry = lhs >= rhs;
    }

    /// Decrements the value in register `reg`, updating `flags`.
    #[inline]
    pub fn decrement(&mut self, reg: Reg, flags: &mut Flags) {
        let value = self.get(reg).wrapping_sub(1);
        self.set(reg, value);
        flags.zero = self.get(reg) == 0;
    }

    /// Returns the value in register `reg`.
    #[inline]
    #[must_use]
    pub fn get(&self, reg: Reg) -> u16 {
        use Reg::*;
        match reg {
            A => u16::from(self.ra),
            B => u16::from(self.rb),
            C => u16::from(self.rc),
            D => u16::from(self.rd),
            E => u16::from(self.re),
            F => u16::from(self.rf),
            G => u16::from(self.rg),
            H => u16::from(self.rh),
            Reg::AB => u16::from_be_bytes([self.ra, self.rb]),
            Reg::CD => u16::from_be_bytes([self.rc, self.rd]),
            Reg::EF => u16::from_be_bytes([self.re, self.rf]),
            Reg::GH => u16::from_be_bytes([self.rg, self.rh]),
        }
    }

    /// Returns the word in register pair `reg`.
    #[inline]
    #[must_use]
    pub fn get16(&self, reg: Reg) -> u16 {
        use Reg::*;
        match reg {
            Reg::AB => u16::from_be_bytes([self.ra, self.rb]),
            Reg::CD => u16::from_be_bytes([self.rc, self.rd]),
            Reg::EF => u16::from_be_bytes([self.re, self.rf]),
            Reg::GH => u16::from_be_bytes([self.rg, self.rh]),
            A | B | C | D | E | F | G | H => unreachable!("called `get16` on 8-bit register {reg}"),
        }
    }

    /// Returns the byte in register `reg`.
    #[inline]
    #[must_use]
    pub fn get8(&self, reg: Reg) -> u8 {
        use Reg::*;
        match reg {
            A => self.ra,
            B => self.rb,
            C => self.rc,
            D => self.rd,
            E => self.re,
            F => self.rf,
            G => self.rg,
            H => self.rh,
            AB | CD | EF | GH => unreachable!("called `get8` on 16-bit register pair {reg}"),
        }
    }

    /// Increments the value in register `reg`, updating `flags`.
    #[inline]
    pub fn increment(&mut self, reg: Reg, flags: &mut Flags) {
        let value = self.get(reg).wrapping_add(1);
        self.set(reg, value);
        flags.zero = self.get(reg) == 0;
    }

    /// Sets register `reg` to the value `val`.
    ///
    /// Flags are not affected.
    #[expect(clippy::as_conversions, reason = "truncation is correct")]
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn load(&mut self, reg: Reg, val: u16) {
        use crate::regs::Reg::*;
        match reg {
            A | B | C | D | E | F | G | H => self.set8(reg, val as u8),
            AB | CD | EF | GH => self.set16(reg, val),
        }
    }

    /// Sets register `reg` to the value `val`.
    #[expect(clippy::as_conversions, reason = "truncation is correct")]
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn set(&mut self, reg: Reg, val: u16) {
        use Reg::*;
        match reg {
            A => self.ra = val as u8,
            B => self.rb = val as u8,
            C => self.rc = val as u8,
            D => self.rd = val as u8,
            E => self.re = val as u8,
            F => self.rf = val as u8,
            G => self.rg = val as u8,
            H => self.rh = val as u8,
            AB => [self.ra, self.rb] = val.to_be_bytes(),
            CD => [self.rc, self.rd] = val.to_be_bytes(),
            EF => [self.re, self.rf] = val.to_be_bytes(),
            GH => [self.rg, self.rh] = val.to_be_bytes(),
        }
    }

    /// Sets register pair `reg` to the word `val`.
    #[inline]
    pub fn set16(&mut self, reg: Reg, val: u16) {
        use Reg::*;
        match reg {
            AB => [self.ra, self.rb] = val.to_be_bytes(),
            CD => [self.rc, self.rd] = val.to_be_bytes(),
            EF => [self.re, self.rf] = val.to_be_bytes(),
            GH => [self.rg, self.rh] = val.to_be_bytes(),
            A | B | C | D | E | F | G | H => unreachable!("called `set16` on 8-bit register {reg}"),
        }
    }

    /// Sets register `reg` to the byte `val`.
    #[inline]
    pub fn set8(&mut self, reg: Reg, val: u8) {
        use Reg::*;
        match reg {
            A => self.ra = val,
            B => self.rb = val,
            C => self.rc = val,
            D => self.rd = val,
            E => self.re = val,
            F => self.rf = val,
            G => self.rg = val,
            H => self.rh = val,
            AB | CD | EF | GH => unreachable!("called `set8` on 16-bit register pair {reg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::regs::{Reg::AB, Reg::*};

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
