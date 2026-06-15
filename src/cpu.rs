use core::fmt::{Display, Formatter};

use crate::{
    bus::{Bus, State},
    device::Device,
    instructions::{INSTRUCTIONS, Instruction},
    regs::Regs,
};
use Phase::{Decode, Execute, FetchOpcode, FetchOperand, ReadOperand, WaitOpcode, WaitOperand};

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
}

impl Device for Cpu {
    /// Performs the current phase, and sets the next phase.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.phase = match self.phase {
            Decode => {
                self.ins = INSTRUCTIONS.get(&bus.data).unwrap_or_default();
                if self.ins.bytes == 2 {
                    FetchOperand
                } else {
                    bus.defer_write(vec![State::Mem(false)]);
                    Execute
                }
            }
            Execute => {
                (self.ins.execute)(self);
                FetchOpcode
            }
            FetchOpcode => {
                if self.halt {
                    FetchOpcode
                } else {
                    bus.defer_write(vec![State::Addr(self.pc), State::Mem(true)]);
                    self.pc = self.pc.wrapping_add(1);
                    WaitOpcode
                }
            }
            FetchOperand => {
                bus.defer_write(vec![State::Addr(self.pc), State::Mem(true)]);
                self.pc = self.pc.wrapping_add(1);
                WaitOperand
            }
            ReadOperand => {
                self.operand = bus.data;
                bus.defer_write(vec![State::Mem(false)]);
                Execute
            }
            WaitOpcode => Decode,
            WaitOperand => ReadOperand,
        }
    }
}

/// The phase of the CPU.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Phase {
    /// Decodes the opcode or operand on the data bus.
    Decode,
    /// Executes the current instruction.
    Execute,
    /// Requests the next opcode from memory.
    #[default]
    FetchOpcode,
    /// Requests a single operand from memory.
    FetchOperand,
    /// Read the operand from the bus.
    ReadOperand,
    /// Wait for the next opcode.
    WaitOpcode,
    /// Wait for the next operand.
    WaitOperand,
}

impl Display for Phase {
    #[expect(clippy::absolute_paths, reason = "disambiguate from anyhow::Result")]
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Decode => "DCOD",
                Execute => "EXEC",
                FetchOpcode => "FTCH",
                FetchOperand => "FOPR",
                ReadOperand => "ROPR",
                WaitOpcode => "WAIT",
                WaitOperand => "WOPR",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions::Opcode::{LdAN, Nop},
        regs::Reg8::A,
    };

    use super::*;
    use Phase::{Decode, Execute, FetchOpcode, FetchOperand, ReadOperand, WaitOpcode, WaitOperand};

    #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
    #[test]
    fn cpu_phases_are_correct_for_zero_operand_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, FetchOpcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, WaitOpcode);
        bus.data = Nop as u8; // nop
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Execute);
    }

    #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
    #[test]
    fn cpu_phases_are_correct_for_one_operand_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, FetchOpcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, WaitOpcode);
        bus.data = LdAN as u8; // ld a, N
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, FetchOperand);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, WaitOperand);
        bus.data = 0xFF;
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, ReadOperand);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Execute);
        cpu.tick(&mut bus);
        assert_eq!(cpu.regs.get8(A), 0xFF);
    }
}
