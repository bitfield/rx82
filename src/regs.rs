use anyhow::bail;

use core::{
    fmt::{Display, Formatter},
    str::FromStr,
};

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
    SP,
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
                Reg::SP => "sp",
            }
        )
    }
}

impl FromStr for Reg {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(value: &str) -> Result<Self, anyhow::Error> {
        Ok(match value {
            "a" => Reg::A,
            "b" => Reg::B,
            "c" => Reg::C,
            "d" => Reg::D,
            "e" => Reg::E,
            "f" => Reg::F,
            "g" => Reg::G,
            "h" => Reg::H,
            "ab" => Reg::AB,
            "cd" => Reg::CD,
            "ef" => Reg::EF,
            "gh" => Reg::GH,
            "sp" => Reg::SP,
            reg => bail!("invalid register {reg}"),
        })
    }
}

impl TryFrom<u8> for Reg {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(id: u8) -> Result<Self, Self::Error> {
        Ok(match id {
            0x00 => Reg::A,
            0x01 => Reg::B,
            0x02 => Reg::C,
            0x03 => Reg::D,
            0x04 => Reg::E,
            0x05 => Reg::F,
            0x06 => Reg::G,
            0x07 => Reg::H,
            0x08 => Reg::AB,
            0x09 => Reg::CD,
            0x0A => Reg::EF,
            0x0B => Reg::GH,
            0x0C => Reg::SP,
            _ => bail!("invalid register id {id:#04X}"),
        })
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
            Reg::SP => 0x0C,
        }
    }
}

impl Reg {
    /// Returns true if `reg` is a 16-bit register pair.
    #[inline]
    #[must_use]
    pub fn is16(&self) -> bool {
        matches!(self, Reg::AB | Reg::CD | Reg::EF | Reg::GH | Reg::SP)
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
    sp: u16,
}

impl Regs {
    /// Returns the value in register `reg`.
    ///
    /// For 8-bit registers, the high byte will always be 0.
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
            Reg::SP => self.sp,
        }
    }

    /// Sets register `reg` to the value `val`.
    ///
    /// For 8-bit registers, the high byte is ignored.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn set(&mut self, reg: Reg, val: u16) -> u16 {
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
            SP => self.sp = val,
        }
        self.get(reg)
    }
}

/// Returns the source and target registers specified by `regs`.
///
/// The source register ID is encoded in the high nibble, the target register in the low
/// nibble. For example, in the instruction `ld a, (cd)` (load register indirect), the
/// source register is `cd` and the target register is `a`. The instruction is followed
/// by an operand byte encoding these registers as 0x91 (9 = 0b1001 = `cd`, 1 = 0b0001 =
/// `a`).
#[inline]
#[must_use]
pub fn source_and_target_from(regs: u8) -> Option<(Reg, Reg)> {
    if let Ok(source) = Reg::try_from((regs & 0x0F0) >> 4_u8)
        && let Ok(target) = Reg::try_from(regs & 0x0F)
    {
        Some((source, target))
    } else {
        None
    }
}

/// Returns the operand byte encoding `source` and `target` registers.
///
/// See [`source_and_target_from`] for details of the encoding.
#[inline]
#[must_use]
pub fn u8_from(source: Reg, target: Reg) -> u8 {
    (u8::from(source) << 4_u8) | u8::from(target)
}

#[cfg(test)]
mod tests {
    use crate::regs::{Reg::AB, Reg::*};

    use super::*;

    #[test]
    fn addressing_individual_regs_works() {
        let mut regs = Regs::default();
        regs.set(A, 0x00FF);
        assert_eq!(regs.get(A), 0x00FF, "wrong A");
        regs.set(B, 0x00FF);
        assert_eq!(regs.get(B), 0x00FF, "wrong B");
    }

    #[test]
    fn addressing_reg_pairs_works() {
        let mut regs = Regs::default();
        regs.set(A, 0x00DE);
        regs.set(B, 0x00AD);
        assert_eq!(regs.get(AB), 0xDEAD, "wrong AB");
        regs.set(AB, 0xBEEF);
        assert_eq!(regs.get(AB), 0xBEEF, "wrong AB");
        assert_eq!(regs.get(A), 0xBE, "wrong A");
        assert_eq!(regs.get(B), 0xEF, "wrong B");
    }

    #[test]
    fn addressing_sp_works() {
        let mut regs = Regs::default();
        let sp = Reg::SP;
        regs.set(sp, 0xFFFF);
        assert!(sp.is16(), "SP should be 16-bit");
        assert_eq!(regs.get(sp), 0xFFFF, "wrong SP");
    }
}
