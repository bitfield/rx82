use anyhow::{Result, ensure};

use std::{collections::HashMap, sync::LazyLock};

use crate::{cpu::Cpu, regs::Reg8, system::System};

/// Temporary opcode constants to make test programs easier to read.
pub const NOP: u8 = 0x00;
pub const LDA_N: u8 = 0x01;
pub const HALT: u8 = 0x02;

/// The instruction set.
pub static INSTRUCTIONS: LazyLock<HashMap<u8, Instruction>> = LazyLock::new(|| {
    HashMap::from([
        (
            NOP,
            Instruction {
                name: "nop",
                bytes: 1,
                execute: |_| {},
                test: |_| Ok(()),
            },
        ),
        (
            HALT,
            Instruction {
                name: "halt",
                bytes: 1,
                execute: |cpu: &mut Cpu| cpu.halt = true,
                test: |sys: &mut System| -> Result<()> {
                    sys.mem.load(0x0000, &[HALT])?;
                    sys.cpu.pc = 0x0000;
                    sys.tick()?; // fetch
                    sys.tick()?; // memwait
                    sys.tick()?; // decode
                    sys.tick()?; // execute
                    ensure!(sys.bus.halt, "/HLT not active");
                    ensure!(sys.cpu.pc == 0x0001, "wrong PC");
                    Ok(())
                },
            },
        ),
        (
            LDA_N,
            Instruction {
                name: "ld a",
                bytes: 2,
                execute: |cpu: &mut Cpu| cpu.regs.set8(Reg8::A, cpu.operand),
                test: |sys: &mut System| -> Result<()> {
                    sys.cpu.regs.set8(Reg8::A, 0x00);
                    sys.mem.load(0x0000, &[LDA_N, 0xFF])?;
                    sys.cpu.pc = 0x0000;
                    sys.tick()?; // fetch
                    sys.tick()?; // memwait
                    sys.tick()?; // decode
                    sys.tick()?; // fetch operand
                    sys.tick()?; // memwait
                    sys.tick()?; // read operand
                    sys.tick()?; // execute
                    ensure!(sys.cpu.regs.get8(Reg8::A) == 0xFF);
                    ensure!(sys.cpu.pc == 0x0002);
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
        let mut sys = System {
            debug: true,
            ..Default::default()
        };
        for (opcode, ins) in INSTRUCTIONS.iter() {
            sys.cpu.halt = false;
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
