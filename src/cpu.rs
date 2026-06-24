use core::fmt::{Display, Formatter};

use crate::{
    instructions::{Instruction, Operands},
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
    pub ins: Instruction,
    /// The current operand (high byte).
    pub op_hi: u8,
    /// The current operand (low byte).
    pub op_lo: u8,
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
    #[expect(clippy::unreachable, reason = "illegal state")]
    /// Performs the current phase, and sets the next phase.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.phase = match self.phase {
            Phase::Decode => match self.target {
                // We've fetched an instruction; what kind is it?
                Target::Opcode => {
                    let opcode = bus.data;
                    self.ins = Instruction::from(opcode);
                    match self.ins.operands() {
                        Operands::Zero => {
                            // No operands needed, so execute it
                            bus.defer_write(vec![State::Mem(false)]);
                            Phase::Execute
                        }
                        Operands::One | Operands::Two => {
                            // 1 or 2 operands needed; fetch the first
                            self.target = Target::Operand;
                            Phase::Fetch
                        }
                    }
                }
                // We've fetched an operand; check if we still need another.
                Target::Operand => {
                    self.op_lo = bus.data;
                    match self.ins.operands() {
                        Operands::Two => {
                            // Yes, fetch the second operand
                            self.target = Target::Operand2;
                            Phase::Fetch
                        }
                        Operands::One => {
                            // No, execute this instruction
                            bus.defer_write(vec![State::Mem(false)]);
                            Phase::Execute
                        }
                        Operands::Zero => {
                            unreachable!("fetched operand for zero-operand instruction")
                        }
                    }
                }
                Target::Operand2 => {
                    self.op_hi = bus.data;
                    bus.defer_write(vec![State::Mem(false)]);
                    Phase::Execute
                }
                Target::Write(_, _) => unreachable!("reached decode phase after memory write"),
            },
            Phase::Execute => {
                // Execute this instruction
                let ins = self.ins.clone();
                ins.execute(self);
                if let Target::Write(addr, val) = self.target {
                    // Write the result
                    bus.defer_write(vec![
                        State::Addr(addr),
                        State::Data(val),
                        State::Mem(true),
                        State::Write(true),
                    ]);
                    Phase::Wait
                } else {
                    // Fetch the next instruction
                    self.target = Target::Opcode;
                    Phase::Fetch
                }
            }
            Phase::Fetch => {
                if self.halt {
                    // Just keep looping in this state
                    Phase::Fetch
                } else {
                    // Issue a memory read and await the result
                    bus.defer_write(vec![
                        State::Addr(self.pc),
                        State::Mem(true),
                        State::Write(false),
                    ]);
                    self.pc = self.pc.wrapping_add(1);
                    Phase::Wait
                }
            }
            Phase::Wait => match self.target {
                Target::Write(_, _) => {
                    self.target = Target::Opcode;
                    Phase::Fetch
                }
                Target::Opcode | Target::Operand | Target::Operand2 => Phase::Decode,
            },
        }
    }
}

impl Cpu {
    /// Resets the CPU to its power-on state: all registers zero, PC zero, not halted,
    /// phase 'fetch' and target 'opcode'.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// The phase the CPU will execute next tick.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Phase {
    /// Reads the opcode or operand from the data bus.
    Decode,
    /// Executes the current instruction.
    Execute,
    /// Requests the next opcode or operand from memory.
    #[default]
    Fetch,
    /// Waits for an in-flight memory request to complete.
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

#[expect(clippy::exhaustive_enums, reason = "this actually is exhaustive")]
/// The target of the next fetch or wait phase.
#[derive(Debug, Default, PartialEq)]
pub enum Target {
    /// An opcode.
    #[default]
    Opcode,
    /// A 1-byte operand (or low byte of a 2-byte operand).
    Operand,
    /// A second 1-byte operand (or high byte of a 2-byte operand).
    Operand2,
    /// Waiting for a memory write to complete.
    Write(u16, u8),
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions::Instruction::*,
        regs::{Reg8::*, Reg16::*},
    };

    use super::*;

    #[test]
    fn cpu_states_are_correct_for_1_byte_instruction() {
        use Instruction::*;
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Opcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = u8::from(Nop);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Execute);
        assert_eq!(cpu.pc, 0x0001);
    }

    #[test]
    fn cpu_states_are_correct_for_2_byte_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Opcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = u8::from(LoadRegImm8(A)); // ld a, N
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
    fn cpu_states_are_correct_for_3_byte_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Opcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = u8::from(LoadRegImm16(AB)); // ld ab, NN
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Operand);
        assert_eq!(cpu.pc, 0x0001);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = 0xEF;
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Operand2);
        assert_eq!(cpu.pc, 0x0002);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = 0xBE;
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Decode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Execute);
        cpu.tick(&mut bus);
        assert_eq!(cpu.regs.get16(AB), 0xBEEF);
        assert_eq!(cpu.pc, 0x0003);
    }

    // #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
    // #[test]
    // fn cpu_states_are_correct_for_mem_write_instruction() {
    //     let mut cpu = Cpu::default();
    //     let mut bus = Bus::default();
    //     cpu.regs.set8(A, 0xFF);
    //     assert_eq!(cpu.phase, Phase::Fetch);
    //     assert_eq!(cpu.target, Target::Opcode);
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Wait);
    //     bus.data = LdMemByteA as u8; // ld (NN), a
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Decode);
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Fetch);
    //     assert_eq!(cpu.target, Target::Operand);
    //     assert_eq!(cpu.pc, 0x0001);
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Wait);
    //     bus.data = 0xEF;
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Decode);
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Fetch);
    //     assert_eq!(cpu.target, Target::Operand2);
    //     assert_eq!(cpu.pc, 0x0002);
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Wait);
    //     bus.data = 0xBE;
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Decode);
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Execute);
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.phase, Phase::Wait);
    //     assert_eq!(cpu.target, Target::Write(0xBEEF, 0xFF));
    //     cpu.tick(&mut bus);
    //     assert_eq!(cpu.pc, 0x0003);
    // }

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
