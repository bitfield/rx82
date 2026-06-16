use anyhow::{Result, ensure};

use std::{collections::HashMap, sync::LazyLock};

use crate::{
    cpu::Cpu,
    regs::{
        Reg8::{A, B},
        Reg16::AB,
    },
    system::System,
};

use Opcode::*;

/// The R8 instruction set.
#[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
pub static INSTRUCTIONS: LazyLock<HashMap<u8, Instruction>> = LazyLock::new(|| {
    HashMap::from([
        (
            Nop as u8,
            Instruction {
                name: "nop",
                length: Length::OneByte,
                execute: |_| {},
                test: |_| Ok(()),
            },
        ),
        (
            Halt as u8,
            Instruction {
                name: "halt",
                length: Length::OneByte,
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
            LdAN as u8,
            Instruction {
                name: "ld a",
                length: Length::TwoBytes,
                execute: |cpu: &mut Cpu| cpu.regs.set8(A, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    sys.run_program(&[LdAN as u8, 0xFF, Halt as u8])?;
                    ensure!(sys.cpu.regs.get8(A) == 0xFF, "wrong A");
                    ensure!(sys.cpu.pc == 0x0003, "wrong PC");
                    Ok(())
                },
            },
        ),
        (
            LdABN as u8,
            Instruction {
                name: "ld ab",
                length: Length::ThreeBytes,
                execute: |cpu: &mut Cpu| {
                    cpu.regs.set8(A, cpu.op_hi);
                    cpu.regs.set8(B, cpu.op_lo);
                },
                test: |sys: &mut System| -> Result<()> {
                    sys.run_program(&[LdABN as u8, 0xEF, 0xBE, Halt as u8])?;
                    ensure!(sys.cpu.regs.get16(AB) == 0xBEEF, "wrong AB");
                    ensure!(sys.cpu.pc == 0x0004, "wrong PC");
                    Ok(())
                },
            },
        ),
        (
            LdBN as u8,
            Instruction {
                name: "ld b",
                length: Length::TwoBytes,
                execute: |cpu: &mut Cpu| cpu.regs.set8(B, cpu.op_lo),
                test: |sys: &mut System| -> Result<()> {
                    sys.run_program(&[LdBN as u8, 0xFF, Halt as u8])?;
                    ensure!(sys.cpu.regs.get8(B) == 0xFF, "wrong B");
                    ensure!(sys.cpu.pc == 0x0003, "wrong PC");
                    Ok(())
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
    pub length: Length,
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
            length: Length::OneByte,
            execute: |_| {},
            test: |_| Ok(()),
        }
    }
}

/// Identifies whether this is a 1, 2, or 3-byte instruction.
#[derive(Clone, Debug, PartialEq)]
pub enum Length {
    OneByte,
    TwoBytes,
    ThreeBytes,
}

/// Identifies the specific opcode.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    /// 'nop'.
    Nop,
    /// 'ld a, N'
    LdAN,
    /// 'ld ab, NN'
    LdABN,
    /// 'ld b, N'
    LdBN,
    /// 'halt'
    Halt,
}

#[cfg(test)]
mod tests {
    use anyhow::Context as _;

    use crate::system::System;

    use super::*;

    #[expect(
        clippy::iter_over_hash_type,
        reason = "order doesn't matter for self-test"
    )]
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
