use anyhow::{Result, bail};

use std::{collections::HashMap, sync::LazyLock};

use crate::{cpu::Cpu, regs::Reg8, system::System};

/// Temporary opcode constants to make test programs easier to read.
pub(crate) const NOP: u8 = 0x00;
pub(crate) const LDA_N: u8 = 0x01;

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
            LDA_N,
            Instruction {
                name: "ld a, N",
                bytes: 2,
                execute: |cpu: &mut Cpu| cpu.regs.set8(Reg8::A, cpu.operand),
                test: |sys: &mut System| -> Result<()> {
                    sys.cpu.regs.set8(Reg8::A, 0x00);
                    sys.mem.load(0x0000, &[LDA_N, 0xFF])?;
                    sys.cpu.pc = 0x0000;
                    sys.tick()?; // fetch
                    sys.debug_print();
                    sys.cpu.debug_print();
                    sys.tick()?; // memwait
                    sys.debug_print();
                    sys.cpu.debug_print();
                    sys.tick()?; // decode
                    sys.debug_print();
                    sys.cpu.debug_print();
                    sys.tick()?; // fetch operand
                    sys.debug_print();
                    sys.cpu.debug_print();
                    sys.tick()?; // memwait
                    sys.debug_print();
                    sys.cpu.debug_print();
                    sys.tick()?; // read operand
                    sys.debug_print();
                    sys.cpu.debug_print();
                    sys.tick()?; // execute
                    sys.debug_print();
                    sys.cpu.debug_print();
                    let val = sys.cpu.regs.get8(Reg8::A);
                    if val != 0xFF {
                        bail!("want A=0xFF, got A={val:02X}")
                    }
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
        let mut sys = System::default();
        for (opcode, ins) in INSTRUCTIONS.iter() {
            (ins.test)(&mut sys)
                .context(format!(
                    "opcode {opcode:#04X} ({}) failed self-test",
                    ins.name
                ))
                .unwrap();
        }
    }
}
