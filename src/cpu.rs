use crate::{
    bus::Bus,
    device::Device,
    instructions::INSTRUCTIONS,
    regs::{Reg8::A, Regs},
};

#[non_exhaustive]
#[derive(Debug, Default)]
pub enum Phase {
    Execute,
    #[default]
    FetchOp,
    MemWait,
}

#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    pub opcode: u8,
    pub pc: u16,
    pub phase: Phase,
    pub regs: Regs,
}

impl Device for Cpu {
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.phase = match self.phase {
            Phase::FetchOp => {
                bus.addr = self.pc;
                bus.mem = true;
                bus.dirty = true;
                Phase::MemWait
            }
            Phase::Execute => {
                self.opcode = bus.data;
                bus.mem = false;
                bus.dirty = true;
                self.pc = self.pc.wrapping_add(1);
                println!(
                    "PC {:04X} A {:02X} OP {:02X}",
                    self.pc,
                    self.regs.get8(A),
                    self.opcode,
                );
                if let Some(ins) = INSTRUCTIONS.get(&self.opcode) {
                    (ins.execute)(self);
                }
                Phase::FetchOp
            }
            Phase::MemWait => Phase::Execute,
        };
    }
}
