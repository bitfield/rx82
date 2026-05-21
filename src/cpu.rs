use crate::{device::Device, instructions::INSTRUCTIONS, memory::Memory, regs::Regs};

#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    pub pc: u16,
    pub regs: Regs,
}

impl Device for Cpu {
    #[inline]
    fn step(&mut self, mem: &mut Memory) {
        let opcode = mem.get(self.pc);
        self.pc = self.pc.wrapping_add(1);
        if let Some(ins) = INSTRUCTIONS.get(&opcode) {
            (ins.execute)(self, mem);
        }
    }
}
