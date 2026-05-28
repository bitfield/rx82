use crate::{
    bus::{Bus, State},
    cpu::Phase::{Execute, FetchOperand},
    device::Device,
    instructions::{INSTRUCTIONS, Instruction},
    phase::Phase,
    regs::{Reg8::A, Regs},
};

/// The system CPU.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    /// The current instruction.
    pub ins: &'static Instruction,
    /// Do we need to fetch an operand?
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
    /// Prints the CPU's state and registers.
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

    /// Decodes the current opcode and decides whether an operand is needed.
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
    use crate::{instructions::LDA_N, phase::Phase};

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
        bus.data = LDA_N; // ld a, N
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
