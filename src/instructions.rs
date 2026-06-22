use crate::{cpu::Cpu, regs::Reg8};

// (
//     LdImmWordAB as u8,
//     Instruction {
//         name: "ld ab",
//         bytes: Three,
//         display: |op_lo, op_hi| {
//             format!("ld ab, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
//         },
//         execute: |cpu: &mut Cpu| {
//             cpu.regs.set8(A, cpu.op_hi);
//             cpu.regs.set8(B, cpu.op_lo);
//         },
//         test: |sys: &mut System| -> Result<()> {
//             test_reg_load_immediate_word(sys, LdImmWordAB, AB)
//         },
//     },
// ),
// (
//     LdImmWordCD as u8,
//     Instruction {
//         name: "ld cd",
//         bytes: Three,
//         display: |op_lo, op_hi| {
//             format!("ld cd, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
//         },
//         execute: |cpu: &mut Cpu| {
//             cpu.regs.set8(C, cpu.op_hi);
//             cpu.regs.set8(D, cpu.op_lo);
//         },
//         test: |sys: &mut System| -> Result<()> {
//             test_reg_load_immediate_word(sys, LdImmWordCD, CD)
//         },
//     },
// ),
// (
//     LdImmWordEF as u8,
//     Instruction {
//         name: "ld ef",
//         bytes: Three,
//         display: |op_lo, op_hi| {
//             format!("ld ef, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
//         },
//         execute: |cpu: &mut Cpu| {
//             cpu.regs.set8(E, cpu.op_hi);
//             cpu.regs.set8(F, cpu.op_lo);
//         },
//         test: |sys: &mut System| -> Result<()> {
//             test_reg_load_immediate_word(sys, LdImmWordEF, EF)
//         },
//     },
// ),
// (
//     LdImmWordGH as u8,
//     Instruction {
//         name: "ld gh",
//         bytes: Three,
//         display: |op_lo, op_hi| {
//             format!("ld gh, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
//         },
//         execute: |cpu: &mut Cpu| {
//             cpu.regs.set8(G, cpu.op_hi);
//             cpu.regs.set8(H, cpu.op_lo);
//         },
//         test: |sys: &mut System| -> Result<()> {
//             test_reg_load_immediate_word(sys, LdImmWordGH, GH)
//         },
//     },
// ),
// (
//     LdMemByteA as u8,
//     Instruction {
//         name: "ld (NN), a",
//         bytes: Three,
//         display: |op_lo, op_hi| {
//             format!("ld ({:#06X}), a", u16::from_le_bytes([op_lo, op_hi]))
//         },

//         execute: |cpu: &mut Cpu| {
//             let addr = u16::from_le_bytes([cpu.op_lo, cpu.op_hi]);
//             cpu.target = Target::Write(addr, cpu.regs.get8(A));
//         },
//         test: |sys: &mut System| -> Result<()> { test_mem_load_byte(sys, LdMemByteA) },
//     },
// ),
//     ])
// });

/// Instruction types.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub enum Instruction {
    Halt,
    Illegal(u8),
    LoadRegImm8(Reg8),
    #[default]
    Nop,
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
            0x1 => LoadRegImm8(Reg8::from(opcode & 0x0F)),
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
            LoadRegImm8(reg) => {
                let mut opcode = 0x10;
                opcode |= u8::from(reg);
                opcode
            }
            Nop => 0x01,
        }
    }
}

impl Instruction {
    #[inline]
    pub fn execute(&self, cpu: &mut Cpu) {
        use Instruction::*;
        match *self {
            Halt => cpu.halt = true,
            Illegal(_) | Nop => {}
            LoadRegImm8(reg) => cpu.regs.set8(reg, cpu.op_lo),
        }
    }

    #[inline]
    #[must_use]
    pub fn operands(&self) -> Operands {
        use Instruction::*;
        match *self {
            Halt | Illegal(_) | Nop => Operands::Zero,
            LoadRegImm8(_) => Operands::One,
        }
    }
}

#[expect(clippy::exhaustive_enums, reason = "this actually is exhaustive")]
/// Identifies whether this is a 1, 2, or 3-byte instruction.
#[derive(Clone, Debug, PartialEq)]
pub enum Operands {
    One,
    Two,
    Zero,
}

// #[expect(clippy::single_call_fn, reason = "temporary scaffolding")]
// #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
// fn test_mem_load_byte(sys: &mut System, opcode: Opcode) -> Result<()> {
//     sys.run_program(&[LdImmByteA as u8, 0xFF, opcode as u8, 0xEF, 0xBE, Halt as u8])?;
//     let val = sys.mem.get(0xBEEF);
//     ensure!(
//         val == 0xFF,
//         "wrong mem value after load mem: want 0xFF, got {val:#04X}"
//     );
//     Ok(())
// }

// #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
// fn test_reg_load_immediate_word(sys: &mut System, opcode: Opcode, reg: Reg16) -> Result<()> {
//     sys.run_program(&[opcode as u8, 0xEF, 0xBE, Halt as u8])?;
//     let val = sys.cpu.regs.get16(reg);
//     ensure!(
//         val == 0xBEEF,
//         "wrong '{reg}' value after load immediate: want 0xBEEF, got {val:#06X}"
//     );
//     Ok(())
// }

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use crate::system::System;

    use super::{Instruction::*, Reg8::*};

    #[test]
    fn halt() {
        let mut sys = System::default();
        sys.run_program(&[u8::from(Halt)]).unwrap();
        assert!(sys.cpu.halt, "not halted");
        assert_eq!(sys.cpu.pc, 0x0001, "wrong PC");
    }

    #[test]
    fn ld_reg_imm8() {
        let mut sys = System::default();
        sys.run_program(&[u8::from(LoadRegImm8(A)), 0xFF, u8::from(Halt)])
            .unwrap();
        assert_eq!(sys.cpu.regs.get8(A), 0xFF, "wrong A");
        assert_eq!(sys.cpu.pc, 0x0003, "wrong PC");
    }

    #[test]
    fn nop() {
        let mut sys = System::default();
        sys.run_program(&[u8::from(Nop), u8::from(Halt)]).unwrap();
        assert_eq!(sys.cpu.pc, 0x0002, "wrong PC");
    }
}
