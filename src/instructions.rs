use anyhow::{Result, ensure};

use std::{collections::HashMap, sync::LazyLock};

use crate::{
    cpu::{Cpu, Target},
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
                display: |_, _| "nop".to_owned(),
                execute: |_| {},
                test: |_| Ok(()),
            },
        ),
        (
            Halt as u8,
            Instruction {
                name: "halt",
                bytes: One,
                display: |_, _| "halt".to_owned(),
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
                display: |op, _| format!("ld a, {op:#04X}"),
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
                display: |op, _| format!("ld b, {op:#04X}"),
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
                display: |op, _| format!("ld c, {op:#04X}"),
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
                display: |op, _| format!("ld d, {op:#04X}"),
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
                display: |op, _| format!("ld e, {op:#04X}"),
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
                display: |op, _| format!("ld f, {op:#04X}"),
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
                display: |op, _| format!("ld g, {op:#04X}"),
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
                display: |op, _| format!("ld h, {op:#04X}"),
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
                display: |op_lo, op_hi| {
                    format!("ld ab, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
                },
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
                display: |op_lo, op_hi| {
                    format!("ld cd, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
                },
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
                display: |op_lo, op_hi| {
                    format!("ld ef, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
                },
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
                display: |op_lo, op_hi| {
                    format!("ld gh, {:#06X}", u16::from_le_bytes([op_lo, op_hi]))
                },
                execute: |cpu: &mut Cpu| {
                    cpu.regs.set8(G, cpu.op_hi);
                    cpu.regs.set8(H, cpu.op_lo);
                },
                test: |sys: &mut System| -> Result<()> {
                    test_reg_load_immediate_word(sys, LdImmWordGH, GH)
                },
            },
        ),
        (
            LdMemByteA as u8,
            Instruction {
                name: "ld (NN), a",
                bytes: Three,
                display: |op_lo, op_hi| {
                    format!("ld ({:#06X}), a", u16::from_le_bytes([op_lo, op_hi]))
                },

                execute: |cpu: &mut Cpu| {
                    let addr = u16::from_le_bytes([cpu.op_lo, cpu.op_hi]);
                    cpu.target = Target::Write(addr, cpu.regs.get8(A));
                },
                test: |sys: &mut System| -> Result<()> { test_mem_load_byte(sys, LdMemByteA) },
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
    /// Format the instruction for display.
    pub display: fn(u8, u8) -> String,
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
            display: |_, _| "nop".to_owned(),
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
    Nop,
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
    LdMemByteA,
    LdMemByteB,
    LdMemByteC,
    LdMemByteD,
    LdMemByteE,
    LdMemByteF,
    LdMemByteG,
    LdMemByteH,
}

#[expect(clippy::single_call_fn, reason = "temporary scaffolding")]
#[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
fn test_mem_load_byte(sys: &mut System, opcode: Opcode) -> Result<()> {
    sys.run_program(&[LdImmByteA as u8, 0xFF, opcode as u8, 0xEF, 0xBE, Halt as u8])?;
    let val = sys.mem.get(0xBEEF);
    ensure!(
        val == 0xFF,
        "wrong mem value after load mem: want 0xFF, got {val:#04X}"
    );
    Ok(())
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
