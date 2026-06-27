use anyhow::anyhow;

use crate::{cpu::Cpu, regs::Reg};

/// Instruction kinds.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub enum InstructionKind {
    Dec(Reg),
    Halt,
    Inc(Reg),
    LoadRegImm(Reg),
    #[default]
    Nop,
    StoreRegDirect(Reg),
}

impl TryFrom<u8> for InstructionKind {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(opcode: u8) -> Result<Self, Self::Error> {
        use InstructionKind::*;
        let reg_id = opcode & 0x0F;
        match opcode {
            0x00 => Ok(Halt),
            0x01 => Ok(Nop),
            0x10..=0x1B => Ok(LoadRegImm(Reg::try_from(reg_id)?)),
            0x20..=0x27 => Ok(StoreRegDirect(Reg::try_from(reg_id)?)),
            0x30..=0x3B => Ok(Inc(Reg::try_from(reg_id)?)),
            0x40..=0x4B => Ok(Dec(Reg::try_from(reg_id)?)),
            _ => Err(anyhow!("invalid opcode {opcode}")),
        }
    }
}

impl From<InstructionKind> for u8 {
    #[inline]
    fn from(ins: InstructionKind) -> Self {
        use InstructionKind::*;
        match ins {
            Dec(reg) => 0x40 | u8::from(reg),
            Halt => 0x00,
            Inc(reg) => 0x30 | u8::from(reg),
            LoadRegImm(reg) => 0x10 | u8::from(reg),
            Nop => 0x01,
            StoreRegDirect(reg) => 0x20 | u8::from(reg),
        }
    }
}

impl InstructionKind {
    /// Executes the instruction.
    #[inline]
    pub fn execute(&self, cpu: &mut Cpu) {
        use crate::regs::Reg::*;
        use InstructionKind::*;
        let op_word = u16::from_le_bytes([cpu.op_lo, cpu.op_hi]);
        match *self {
            Dec(reg) => match reg {
                A | B | C | D | E | F | G | H => {
                    cpu.regs.set8(reg, cpu.regs.get8(reg).wrapping_sub(1));
                    cpu.update_zero_flag(reg);
                }
                AB | CD | EF | GH => {
                    cpu.regs.set16(reg, cpu.regs.get16(reg).wrapping_sub(1));
                    cpu.update_zero_flag(reg);
                }
            },
            Halt => cpu.halt = true,
            Inc(reg) => match reg {
                A | B | C | D | E | F | G | H => {
                    cpu.regs.set8(reg, cpu.regs.get8(reg).wrapping_add(1));
                    cpu.update_zero_flag(reg);
                }
                AB | CD | EF | GH => {
                    cpu.regs.set16(reg, cpu.regs.get16(reg).wrapping_add(1));
                    cpu.update_zero_flag(reg);
                }
            },
            LoadRegImm(reg) => match reg {
                A | B | C | D | E | F | G | H => cpu.regs.set8(reg, cpu.op_lo),
                AB | CD | EF | GH => cpu.regs.set16(reg, op_word),
            },
            Nop => {}
            StoreRegDirect(reg) => cpu.write_mem(op_word, reg),
        }
    }

    /// Returns the number of operands this instruction takes.
    #[inline]
    #[must_use]
    pub fn operands(&self) -> Operands {
        use crate::regs::Reg::*;
        use InstructionKind::*;
        match *self {
            Dec(_) | Halt | Inc(_) | Nop => Operands::Zero,
            LoadRegImm(reg) => match reg {
                A | B | C | D | E | F | G | H => Operands::One,
                AB | CD | EF | GH => Operands::Two,
            },
            StoreRegDirect(_) => Operands::Two,
        }
    }
}

#[expect(clippy::exhaustive_enums, reason = "this actually is exhaustive")]
/// Specifies whether an instruction takes zero, one, or two operands.
#[derive(Clone, Debug, PartialEq)]
pub enum Operands {
    One,
    Two,
    Zero,
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use crate::{asm::asm, regs::Reg::*, system::System};

    #[test]
    fn dec() {
        let mut sys = System::default();
        sys.run_program(&asm("dec ef")).unwrap();
        assert_eq!(sys.cpu.regs.get16(EF), 0xFFFF, "wrong EF");
    }

    #[test]
    fn halt() {
        let mut sys = System::default();
        sys.run_program(&asm("halt")).unwrap();
        assert!(sys.cpu.halt, "not halted");
        assert_eq!(sys.cpu.pc, 0x0001, "wrong PC");
    }

    #[test]
    fn inc() {
        let mut sys = System::default();
        sys.run_program(&asm("inc d")).unwrap();
        assert_eq!(sys.cpu.regs.get8(D), 0x01, "wrong D");
    }

    #[test]
    fn ld_reg_imm8() {
        let mut sys = System::default();
        sys.run_program(&asm("ld a, 0xFF")).unwrap();
        assert_eq!(sys.cpu.regs.get8(A), 0xFF, "wrong A");
        assert_eq!(sys.cpu.pc, 0x0003, "wrong PC");
    }

    #[test]
    fn ld_reg_imm16() {
        let mut sys = System::default();
        sys.run_program(&asm("ld ab, 0x00C0")).unwrap();
        assert_eq!(sys.cpu.regs.get16(AB), 0x00C0, "wrong AB");
        assert_eq!(sys.cpu.pc, 0x0004, "wrong PC");
    }

    #[test]
    fn nop() {
        let mut sys = System::default();
        sys.run_program(&asm("nop")).unwrap();
        assert_eq!(sys.cpu.pc, 0x0002, "wrong PC");
    }

    #[test]
    fn store_reg_direct() {
        let mut sys = System::default();
        sys.cpu.regs.set8(A, 0xFF);
        sys.run_program(&asm("ld 0xBEEF, a")).unwrap();
        let val = sys.mem.get(0xBEEF);
        assert_eq!(val, 0xFF, "wrong mem value");
    }

    #[expect(clippy::bool_assert_comparison, reason = "clarity")]
    #[test]
    fn zero_flag() {
        use crate::cpu::Flag::*;
        let mut sys = System::default();
        assert_eq!(sys.cpu.flag(Zero), false, "zero flag wrongly initialised");
        sys.run_program(&asm("dec a")).unwrap(); // a = -1
        assert_eq!(sys.cpu.flag(Zero), false, "zero flag set after dec");
        sys.run_program(&asm("inc a")).unwrap(); // a = 0
        assert_eq!(sys.cpu.flag(Zero), true, "zero flag clear after inc");
        sys.run_program(&asm("inc a")).unwrap(); // a = 1
        assert_eq!(sys.cpu.flag(Zero), false, "zero flag set after inc");
        sys.run_program(&asm("dec a")).unwrap(); // a = 0
        assert_eq!(sys.cpu.flag(Zero), true, "zero flag clear after dec");
    }
}
