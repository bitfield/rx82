use crate::{
    cpu::Cpu,
    regs::{Reg8, Reg16},
};

/// Instruction types.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub enum Instruction {
    Halt,
    Illegal(u8),
    LoadRegImm16(Reg16),
    LoadRegImm8(Reg8),
    #[default]
    Nop,
    StoreRegDirect(Reg8),
}

impl From<u8> for Instruction {
    #[inline]
    fn from(opcode: u8) -> Self {
        use Instruction::*;
        match opcode >> 4 {
            0x0 => match opcode & 0x0F {
                0x0 => Halt,
                0x1 => Nop,
                _ => Illegal(opcode),
            },
            0x1 => match opcode & 0x0F {
                0x00..=0x07 => LoadRegImm8(Reg8::from(opcode & 0x0F)),
                0x08..=0x0B => LoadRegImm16(Reg16::from(opcode & 0x0F)),
                _ => Illegal(opcode),
            },
            0x2 => match opcode & 0x0F {
                0x00..=0x07 => StoreRegDirect(Reg8::from(opcode & 0x0F)),
                _ => Illegal(opcode),
            },
            _ => Illegal(opcode),
        }
    }
}

impl From<Instruction> for u8 {
    #[inline]
    fn from(ins: Instruction) -> Self {
        use Instruction::*;
        match ins {
            Halt => 0x00,
            Illegal(opcode) => opcode,
            LoadRegImm8(reg) => 0x10 | u8::from(reg),
            LoadRegImm16(reg) => 0x10 | u8::from(reg),
            Nop => 0x01,
            StoreRegDirect(reg) => 0x20 | u8::from(reg),
        }
    }
}

impl Instruction {
    #[inline]
    pub fn execute(&self, cpu: &mut Cpu) {
        use crate::cpu::Target;
        use Instruction::*;
        let op_word = u16::from_le_bytes([cpu.op_lo, cpu.op_hi]);
        match *self {
            Halt => cpu.halt = true,
            Illegal(_) | Nop => {}
            LoadRegImm8(reg) => cpu.regs.set8(reg, cpu.op_lo),
            LoadRegImm16(reg) => cpu.regs.set16(reg, op_word),
            StoreRegDirect(reg) => {
                cpu.target = Target::Write(op_word, cpu.regs.get8(reg));
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn operands(&self) -> Operands {
        use Instruction::*;
        match *self {
            Halt | Illegal(_) | Nop => Operands::Zero,
            LoadRegImm8(_) => Operands::One,
            LoadRegImm16(_) | StoreRegDirect(_) => Operands::Two,
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
    use crate::{
        regs::{Reg8::*, Reg16::*},
        system::System,
    };

    use super::Instruction::*;

    #[test]
    fn halt() {
        let mut sys = System::default();
        sys.run_program(&[
            u8::from(Halt), // halt
        ])
        .unwrap();
        assert!(sys.cpu.halt, "not halted");
        assert_eq!(sys.cpu.pc, 0x0001, "wrong PC");
    }

    #[test]
    fn ld_reg_imm8() {
        let mut sys = System::default();
        sys.run_program(&[
            u8::from(LoadRegImm8(A)),
            0xFF,           // ld a, 0xFF
            u8::from(Halt), // halt
        ])
        .unwrap();
        assert_eq!(sys.cpu.regs.get8(A), 0xFF, "wrong A");
        assert_eq!(sys.cpu.pc, 0x0003, "wrong PC");
    }

    #[test]
    fn ld_reg_imm16() {
        let mut sys = System::default();
        sys.run_program(&[
            u8::from(LoadRegImm16(AB)),
            0x00C0,         // ld ab, 0x00C0
            u8::from(Halt), // halt
        ])
        .unwrap();
        assert_eq!(sys.cpu.regs.get16(AB), 0x00C0, "wrong AB");
        assert_eq!(sys.cpu.pc, 0x0004, "wrong PC");
    }

    #[test]
    fn nop() {
        let mut sys = System::default();
        sys.run_program(&[
            u8::from(Nop),  // nop
            u8::from(Halt), // halt
        ])
        .unwrap();
        assert_eq!(sys.cpu.pc, 0x0002, "wrong PC");
    }

    #[test]
    fn store_reg_direct() {
        let mut sys = System::default();
        sys.cpu.regs.set8(A, 0xFF);
        sys.run_program(&[
            u8::from(StoreRegDirect(A)),
            0xEF,
            0xBE,           // ld 0xBEEF, a
            u8::from(Halt), // halt
        ])
        .unwrap();
        let val = sys.mem.get(0xBEEF);
        assert_eq!(val, 0xFF, "wrong mem value");
    }
}
