use anyhow::{Result, ensure};

use std::{collections::HashMap, sync::LazyLock};

use crate::{
    cpu::Cpu,
    regs::{
        Reg8::{self, *},
        Reg16::{self, *},
    },
    system::System,
};

use Length::*;
use Opcode::*;

/// The R8 instruction set.
#[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
pub static INSTRUCTIONS: LazyLock<HashMap<u8, Instruction>> = LazyLock::new(|| {
    HashMap::from([
        (
            Nop as u8,
            Instruction {
                name: "nop",
                bytes: One,
                execute: |_| {},
                test: |_| Ok(()),
            },
        ),
        (
            Halt as u8,
            Instruction {
                name: "halt",
                bytes: One,
                execute: |cpu: &mut Cpu| cpu.halt = true,
                test: |sys: &mut System| -> Result<()> {
                    sys.run_program(&[Halt as u8])?;
                    ensure!(sys.cpu.halt, "not halted");
                    ensure!(sys.cpu.pc == 0x0001, "wrong PC");
                    Ok(())
                },
            },
        ),
        (
            LdImmByteA as u8,
            Instruction {
                name: "ld a",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(A, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteA, A)
                },
            },
        ),
        (
            LdImmByteB as u8,
            Instruction {
                name: "ld b",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(B, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteB, B)
                },
            },
        ),
        (
            LdImmByteC as u8,
            Instruction {
                name: "ld c",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(C, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteC, C)
                },
            },
        ),
        (
            LdImmByteD as u8,
            Instruction {
                name: "ld d",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(D, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteD, D)
                },
            },
        ),
        (
            LdImmByteE as u8,
            Instruction {
                name: "ld e",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(E, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteE, E)
                },
            },
        ),
        (
            LdImmByteF as u8,
            Instruction {
                name: "ld f",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(F, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteF, F)
                },
            },
        ),
        (
            LdImmByteG as u8,
            Instruction {
                name: "ld g",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(G, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteG, G)
                },
            },
        ),
        (
            LdImmByteH as u8,
            Instruction {
                name: "ld h",
                bytes: Two,
                execute: |cpu: &mut Cpu| cpu.regs.set8(H, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_byte(sys, LdImmByteH, H)
                },
            },
        ),
        (
            LdImmWordAB as u8,
            Instruction {
                name: "ld ab",
                bytes: Three,
                execute: |cpu: &mut Cpu| {
                    cpu.regs.set8(A, cpu.op_hi);
                    cpu.regs.set8(B, cpu.op_lo);
                },
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_word(sys, LdImmWordAB, AB)
                },
            },
        ),
        (
            LdImmWordCD as u8,
            Instruction {
                name: "ld cd",
                bytes: Three,
                execute: |cpu: &mut Cpu| {
                    cpu.regs.set8(C, cpu.op_hi);
                    cpu.regs.set8(D, cpu.op_lo);
                },
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_word(sys, LdImmWordCD, CD)
                },
            },
        ),
        (
            LdImmWordEF as u8,
            Instruction {
                name: "ld ef",
                bytes: Three,
                execute: |cpu: &mut Cpu| {
                    cpu.regs.set8(E, cpu.op_hi);
                    cpu.regs.set8(F, cpu.op_lo);
                },
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_word(sys, LdImmWordEF, EF)
                },
            },
        ),
        (
            LdImmWordGH as u8,
            Instruction {
                name: "ld gh",
                bytes: Three,
                execute: |cpu: &mut Cpu| {
                    cpu.regs.set8(G, cpu.op_hi);
                    cpu.regs.set8(H, cpu.op_lo);
                },
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_word(sys, LdImmWordGH, GH)
                },
            },
        ),
    ])
});

/// An instruction definition.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Instruction {
    /// Number of bytes the instruction requires in memory.
    pub bytes: Length,
    /// The closure that executes this instruction.
    pub execute: fn(&mut Cpu),
    /// The instruction's symbolic name.
    pub name: &'static str,
    /// The self-test closure for the instruction (run by `cargo test`).
    pub test: fn(&mut System) -> Result<()>,
}

impl Default for &Instruction {
    /// The default instruction (used in place of unknown opcodes) is `nop`.
    #[inline]
    fn default() -> Self {
        &Instruction {
            name: "nop",
            bytes: One,
            execute: |_| {},
            test: |_| Ok(()),
        }
    }
}

#[expect(clippy::exhaustive_enums, reason = "this actually is exhaustive")]
/// Identifies whether this is a 1, 2, or 3-byte instruction.
#[derive(Clone, Debug, PartialEq)]
pub enum Length {
    One,
    Three,
    Two,
}

/// Identifies the specific opcode.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Halt,
    LdImmByteA,
    LdImmByteB,
    LdImmByteC,
    LdImmByteD,
    LdImmByteE,
    LdImmByteF,
    LdImmByteG,
    LdImmByteH,
    LdImmWordAB,
    LdImmWordCD,
    LdImmWordEF,
    LdImmWordGH,
    Nop,
}

#[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
fn test_reg_load_immediate_byte(sys: &mut System, opcode: Opcode, reg: Reg8) -> Result<()> {
    sys.run_program(&[opcode as u8, 0xFF, Halt as u8])?;
    let val = sys.cpu.regs.get8(reg);
    ensure!(
        val == 0xFF,
        "wrong '{reg}' value after load immediate: want 0xFF, got {val:#04X}"
    );
    Ok(())
}

#[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
fn test_reg_load_immediate_word(sys: &mut System, opcode: Opcode, reg: Reg16) -> Result<()> {
    sys.run_program(&[opcode as u8, 0xEF, 0xBE, Halt as u8])?;
    let val = sys.cpu.regs.get16(reg);
    ensure!(
        val == 0xBEEF,
        "wrong '{reg}' value after load immediate: want 0xBEEF, got {val:#06X}"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use anyhow::Context as _;

    use crate::system::System;

    use super::*;

    #[expect(clippy::iter_over_hash_type, reason = "order doesn't matter")]
    #[expect(clippy::unwrap_used, reason = "test")]
    #[test]
    fn instructions_pass_self_test() {
        let mut sys = System::default();
        for (opcode, ins) in INSTRUCTIONS.iter() {
            sys.cpu.reset();
            (ins.test)(&mut sys)
                .context(format!(
                    "opcode {opcode:#04X} ({}) failed self-test",
                    ins.name
                ))
                .inspect_err(|_| sys.trace())
                .unwrap();
        }
    }
}
