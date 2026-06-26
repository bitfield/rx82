use anyhow::anyhow;

use crate::{cpu::Cpu, regs::Reg};

/// Instruction kinds.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub enum InstructionKind {
    Halt,
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
            _ => Err(anyhow!("invalid opcode {opcode}")),
        }
    }
}

impl From<InstructionKind> for u8 {
    #[inline]
    fn from(ins: InstructionKind) -> Self {
        use InstructionKind::*;
        match ins {
            Halt => 0x00,
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
            Halt => cpu.halt = true,
            Nop => {}
            LoadRegImm(reg) => match reg {
                A | B | C | D | E | F | G | H => cpu.regs.set8(reg, cpu.op_lo),
                AB | CD | EF | GH => cpu.regs.set16(reg, op_word),
            },
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
            Halt | Nop => Operands::Zero,
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
    fn halt() {
        let mut sys = System::default();
        sys.run_program(&asm("halt")).unwrap();
        assert!(sys.cpu.halt, "not halted");
        assert_eq!(sys.cpu.pc, 0x0001, "wrong PC");
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
}
