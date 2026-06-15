use anyhow::{Result, ensure};

use std::{collections::HashMap, sync::LazyLock};

use crate::{
    cpu::Cpu,
    instructions::Opcode::Nop,
    regs::Reg8::{A, B},
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
                bytes: 1,
                execute: |_| {},
                test: |_| Ok(()),
            },
        ),
        (
            Halt as u8,
            Instruction {
                name: "halt",
                bytes: 1,
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
                bytes: 2,
                execute: |cpu: &mut Cpu| cpu.regs.set8(A, cpu.operand),
                test: |sys: &mut System| -> Result<()> {
                    sys.run_program(&[LdAN as u8, 0xFF, Halt as u8])?;
                    ensure!(sys.cpu.regs.get8(A) == 0xFF, "wrong A");
                    ensure!(sys.cpu.pc == 0x0003, "wrong PC");
                    Ok(())
                },
            },
        ),
        (
            LdBN as u8,
            Instruction {
                name: "ld b",
                bytes: 2,
                execute: |cpu: &mut Cpu| cpu.regs.set8(B, cpu.operand),
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
    pub bytes: u8,
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
            bytes: 1,
            execute: |_| {},
            test: |_| Ok(()),
        }
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Opcode {
    Nop,
    LdAN,
    LdBN,
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
