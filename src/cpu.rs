use crate::{
    bus::{Bus, State},
    device::Device,
    instructions::{INSTRUCTIONS, Instruction},
    phase::Phase,
    regs::Regs,
};

/// The system CPU.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    /// The current instruction.
    pub ins: &'static Instruction,
    /// Have we just completed an instruction?
    pub instruction_complete: bool,
    /// Are we waiting for an operand to complete this instrution?
    pub need_operand: bool,
    /// The current opcode.
    pub opcode: u8,
    /// The current operand.
    pub operand: u8,
    /// The program counter.
    pub pc: u16,
    /// The current phase.
    pub phase: Phase,
    /// The CPU's registers.
    pub regs: Regs,
}

impl Device for Cpu {
    /// Performs the current phase, and set the next phase.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.instruction_complete = false;
        self.phase = match self.phase {
            Phase::Decode if self.need_operand => {
                self.operand = bus.data;
                bus.defer_write(vec![State::Mem(false)]);
                Phase::Execute
            }
            Phase::Decode => {
                self.opcode = bus.data;
                self.ins = INSTRUCTIONS.get(&self.opcode).unwrap_or_default();
                if self.ins.bytes == 2 {
                    self.need_operand = true;
                    Phase::Fetch
                } else {
                    self.need_operand = false;
                    bus.defer_write(vec![State::Mem(false)]);
                    Phase::Execute
                }
            }
            Phase::Execute => {
                if let Some(ins) = INSTRUCTIONS.get(&self.opcode) {
                    (ins.execute)(self);
                }
                self.instruction_complete = true;
                self.need_operand = false;
                Phase::Fetch
            }
            Phase::Fetch => {
                bus.defer_write(vec![State::Addr(self.pc), State::Mem(true)]);
                self.pc = self.pc.wrapping_add(1);
                Phase::MemWait
            }
            Phase::MemWait => Phase::Decode,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{instructions::LDA_N, phase::Phase};

    use super::*;

    #[test]
    fn cpu_phases_are_correct_for_zero_operand_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::Fetch);
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
        assert_eq!(cpu.phase, Phase::Fetch);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::MemWait);
        bus.data = LDA_N; // ld a, N
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Fetch);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::MemWait);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Execute);
    }
}
