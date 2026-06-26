use core::fmt::{Display, Formatter};

use crate::{
    instructions::{
        InstructionKind::{self, Nop},
        Operands,
    },
    regs::{Reg, Regs},
    system::{Bus, Device, State},
};

/// The system CPU.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    /// HALT flag.
    pub halt: bool,
    /// The current instruction.
    pub ins: InstructionKind,
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
    /// Performs the current phase, and sets the next phase.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.phase = match self.phase {
            Phase::Decode => self.decode(bus),
            Phase::Execute => self.execute(bus),
            Phase::Fetch => self.fetch(bus),
            Phase::Wait => self.wait(),
        }
    }
}

impl Cpu {
    /// Performs the 'decode' phase.
    ///
    /// If the decoded value is an instruction that has no operands, the next phase will be 'execute'.
    /// If the instruction needs operands, the next phase will be 'fetch'.
    ///
    /// If the decoded value is an operand, and no more operands are needed, the next
    /// phase will be 'execute'. If more operands are needed, the next phase will be
    /// 'fetch'.
    #[inline]
    pub fn decode(&mut self, bus: &mut Bus) -> Phase {
        match self.target {
            // We've fetched an instruction; what kind is it?
            Target::Opcode => {
                let opcode = bus.data;
                self.ins = InstructionKind::try_from(opcode).unwrap_or(Nop);
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
        }
    }

    /// Performs the 'execute' phase.
    ///
    /// If the instruction is a memory write, the next phase will be 'wait'. Otherwise
    /// the next phase will be 'fetch'.
    #[inline]
    pub fn execute(&mut self, bus: &mut Bus) -> Phase {
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

    /// Performs the 'fetch' phase.
    ///
    /// If the CPU is halted, the next phase will be 'fetch'. Otherwise, a memory
    /// request is asserted to the bus, and the next phase will be 'wait'.
    #[inline]
    pub fn fetch(&mut self, bus: &mut Bus) -> Phase {
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

    /// Resets the CPU to its power-on state: all registers zero, PC zero, not halted,
    /// phase 'fetch' and target 'opcode'.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Performs the 'wait for memory' phase.
    ///
    /// If the current target is an opcode or operand, the next phase will be 'decode'.
    /// If the target is a memory write, the next phase will be 'fetch'.
    #[inline]
    pub fn wait(&mut self) -> Phase {
        match self.target {
            Target::Write(_, _) => {
                self.target = Target::Opcode;
                Phase::Fetch
            }
            Target::Opcode | Target::Operand | Target::Operand2 => Phase::Decode,
        }
    }

    /// Sets the next sequencer target to write `reg` to memory at `addr`.
    #[inline]
    pub fn write_mem(&mut self, addr: u16, reg: Reg) {
        self.target = Target::Write(addr, self.regs.get8(reg));
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
    use crate::{instructions::InstructionKind::*, regs::Reg::*};

    use super::*;

    #[test]
    fn cpu_states_are_correct_for_1_byte_instruction() {
        use InstructionKind::*;
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
        bus.data = u8::from(LoadRegImm(A)); // ld a, N
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
        bus.data = u8::from(LoadRegImm(AB)); // ld ab, NN
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

    #[test]
    fn cpu_states_are_correct_for_mem_write_instruction() {
        let mut cpu = Cpu::default();
        let mut bus = Bus::default();
        cpu.regs.set8(A, 0xFF);
        assert_eq!(cpu.phase, Phase::Fetch);
        assert_eq!(cpu.target, Target::Opcode);
        cpu.tick(&mut bus);
        assert_eq!(cpu.phase, Phase::Wait);
        bus.data = u8::from(StoreRegDirect(A)); // ld NN, a
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
        assert_eq!(cpu.phase, Phase::Wait);
        assert_eq!(cpu.target, Target::Write(0xBEEF, 0xFF));
        cpu.tick(&mut bus);
        assert_eq!(cpu.pc, 0x0003);
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
