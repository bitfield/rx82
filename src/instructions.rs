use std::{collections::HashMap, sync::LazyLock};

use crate::{cpu::Cpu, memory::Memory, regs::Reg8::A};

pub static INSTRUCTIONS: LazyLock<HashMap<u8, &Instruction>> = LazyLock::new(|| {
    HashMap::from([(
        0x10,
        &Instruction {
            name: "ld a, N",
            execute: |cpu, mem| {
                cpu.regs.set8(A, mem.get(cpu.pc));
                cpu.pc = cpu.pc.wrapping_add(1);
            },
        },
    )])
});

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Instruction {
    pub execute: fn(&mut Cpu, &mut Memory),
    pub name: &'static str,
}
