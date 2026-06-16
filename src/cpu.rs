use core::fmt::{Display, Formatter};

use crate::{
    instructions::{INSTRUCTIONS, Instruction},
    regs::Regs,
    system::{Bus, Device, State},
};

/// The system CPU.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    /// HALT flag.
    pub halt: bool,
    /// The current instruction.
    pub ins: &'static Instruction,
    /// The current operand.
    pub operand: u8,
    /// The program counter.
    pub pc: u16,
    /// The current phase.
    pub phase: Phase,
    /// The CPU's registers.
    pub regs: Regs,
    /// Target of the current memory fetch.
    pub target: Target,
}

impl Device for Cpu {
    /// Performs the current phase, and sets the next phase.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.phase = match self.phase {
            Phase::Decode => match self.target {
                Target::Opcode => {
                    self.ins = INSTRUCTIONS.get(&bus.data).unwrap_or_default();
                    if self.ins.bytes == 2 {
                        self.target = Target::Operand;
                        Phase::Fetch
                    } else {
                        bus.defer_write(vec![State::Mem(false)]);
                        Phase::Execute
                    }
                }
                Target::Operand => {
                    self.operand = bus.data;
                    bus.defer_write(vec![State::Mem(false)]);
                    Phase::Execute
                }
            },
            Phase::Execute => {
                (self.ins.execute)(self);
                self.target = Target::Opcode;
                Phase::Fetch
            }
            Phase::Fetch => {
                if self.halt {
                    Phase::Fetch
                } else {
                    bus.defer_write(vec![State::Addr(self.pc), State::Mem(true)]);
                    self.pc = self.pc.wrapping_add(1);
                    Phase::Wait
                }
            }
            Phase::Wait => Phase::Decode,
        }
    }
}

impl Cpu {
    /// Resets the CPU to its power-on state.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The phase the CPU will execute next tick.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Phase {
    /// Read the opcode or operand from the data bus.
    Decode,
    /// Executes the current instruction.
    Execute,
    /// Requests the next opcode from memory.
    #[default]
    Fetch,
    /// Wait for the next opcode.
    Wait,
}

impl Display for Phase {
    #[expect(clippy::absolute_paths, reason = "disambiguate from anyhow::Result")]
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Phase::Decode => "DCOD",
                Phase::Execute => "EXEC",
                Phase::Fetch => "FTCH",
                Phase::Wait => "WAIT",
            }
        )
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum Target {
    #[default]
    Opcode,
    Operand,
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions::Opcode::{LdAN, Nop},
        regs::{Reg8::A, Reg16::AB},
    };

    use super::*;

    #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
    #[test]
    fn cpu_states_are_correct_for_1_byte_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Opcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = Nop as u8; // nop
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Execute);
        assert_eq!(cpu.pc, 0x0001);
    }

    #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
    #[test]
    fn cpu_states_are_correct_for_2_byte_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Opcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = LdAN as u8; // ld a, N
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Operand);
        assert_eq!(cpu.pc, 0x0001);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = 0xFF;
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Execute);
        cpu.tick(&mut bus);
        assert_eq!(cpu.regs.get8(A), 0xFF);
        assert_eq!(cpu.pc, 0x0002);
    }

    #[test]
    fn reset_resets_cpu() {
        let mut cpu = Cpu::default();
        cpu.regs.set16(AB, 0xBEEF);
        cpu.pc = 0xC000;
        cpu.reset();
        assert_eq!(cpu.regs.get16(AB), 0x0000, "AB not reset");
        assert_eq!(cpu.pc, 0x0000, "PC not reset");
    }
}
