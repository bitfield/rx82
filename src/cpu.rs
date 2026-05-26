use crate::{
    bus::{Bus, State},
    cpu::Phase::{Execute, FetchOperand},
    device::Device,
    instructions::{INSTRUCTIONS, Instruction},
    regs::{Reg8::A, Regs},
};

#[non_exhaustive]
#[derive(Debug, Default, PartialEq)]
pub enum Phase {
    Decode,
    Execute,
    #[default]
    FetchOpcode,
    FetchOperand,
    MemWait,
    ReadOperand,
}

#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    pub ins: &'static Instruction,
    pub need_operand: bool,
    pub opcode: u8,
    pub operand: u8,
    pub pc: u16,
    pub phase: Phase,
    pub regs: Regs,
}

impl Device for Cpu {
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.phase = match self.phase {
            Phase::Decode => {
                self.opcode = bus.data;
                bus.defer_write(vec![State::Mem(false)]);
                self.pc = self.pc.wrapping_add(1);
                self.decode()
            }
            Phase::Execute => {
                self.need_operand = false;
                println!("execute");
                self.debug_print();
                if let Some(ins) = INSTRUCTIONS.get(&self.opcode) {
                    (ins.execute)(self);
                }
                Phase::FetchOpcode
            }
            Phase::FetchOpcode => {
                bus.defer_write(vec![State::Addr(self.pc), State::Mem(true)]);
                Phase::MemWait
            }
            Phase::FetchOperand => {
                self.operand = bus.data;
                bus.defer_write(vec![State::Addr(self.pc), State::Mem(true)]);
                self.need_operand = true;
                Phase::MemWait
            }
            Phase::MemWait => {
                if self.need_operand {
                    Phase::ReadOperand
                } else {
                    Phase::Decode
                }
            }
            Phase::ReadOperand => {
                self.operand = bus.data;
                bus.defer_write(vec![State::Mem(false)]);
                Phase::Execute
            }
        };
    }
}

impl Cpu {
    #[expect(clippy::use_debug, reason = "it's for debugging")]
    #[inline]
    pub fn debug_print(&self) {
        println!(
            "PC {:04X} A {:02X} OC {:02X} O1 {:02X} Phase {:?}",
            self.pc,
            self.regs.get8(A),
            self.opcode,
            self.operand,
            self.phase,
        );
    }

    fn decode(&mut self) -> Phase {
        println!("decoding {:02X}", self.opcode);
        self.ins = INSTRUCTIONS.get(&self.opcode).unwrap_or_default();
        match self.ins.bytes {
            2 => FetchOperand,
            _ => Execute,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_phases_are_correct_for_zero_operand_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::FetchOpcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::MemWait);
        bus.data = 0x00; // nop
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Execute);
    }

    #[test]
    fn cpu_phases_are_correct_for_one_operand_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::FetchOpcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::MemWait);
        bus.data = 0x01; // ld a, N
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::FetchOperand);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::MemWait);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::ReadOperand);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Execute);
    }
}
