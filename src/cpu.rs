use core::fmt::{Display, Formatter};

use crate::{
    instructions::{InstructionKind, Operands},
    regs::{Reg, Regs, source_and_target_from},
    system::{Bus, Device, State},
};

/// The system CPU.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    /// Flags.
    pub flags: Flags,
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
    /// Branches to PC+`dis`.
    #[expect(clippy::cast_possible_wrap, reason = "i8 to u16 is sound")]
    #[expect(clippy::cast_sign_loss, reason = "okay with wrapping_add")]
    #[inline]
    pub fn branch(&mut self, dis: u8) {
        self.pc = self.pc.wrapping_add(dis as i8 as u16); // sign-extend displacement
    }

    /// Performs the 'decode' phase.
    #[inline]
    pub fn decode(&mut self, bus: &mut Bus) -> Phase {
        match self.target {
            Target::Opcode => {
                // We've fetched an instruction; does it need operands?
                let opcode = bus.data;
                self.ins = InstructionKind::try_from(opcode).unwrap_or(InstructionKind::Nop);
                match self.ins.operands() {
                    Operands::Zero => {
                        // No operands needed, go straight to 'execute'
                        bus.pending_write.get_or_insert(vec![State::Mem(false)]);
                        Phase::Execute
                    }
                    Operands::One | Operands::Two => {
                        // 1 or 2 operands needed; fetch the first
                        self.target = Target::Operand;
                        Phase::Fetch
                    }
                }
            }
            Target::Operand => {
                // We've fetched an operand; does the current instruction need one more?
                self.op_hi = 0;
                self.op_lo = bus.data;
                match self.ins.operands() {
                    Operands::Two => {
                        // Yes, fetch the second operand
                        self.target = Target::Operand2;
                        Phase::Fetch
                    }
                    Operands::One => {
                        // No, go to 'execute'
                        bus.pending_write.get_or_insert(vec![State::Mem(false)]);
                        Phase::Execute
                    }
                    Operands::Zero => {
                        unreachable!("fetched operand for zero-operand instruction")
                    }
                }
            }
            Target::Operand2 => {
                // We've fetched the 2nd operand of this instruction; go to 'execute'
                self.op_hi = bus.data;
                bus.pending_write.get_or_insert(vec![State::Mem(false)]);
                Phase::Execute
            }
            Target::Read(_, reg) => {
                // We've fetched a value from memory; load it to the target register
                self.regs.set(reg, u16::from(bus.data));
                // Fetch the next instruction
                self.target = Target::Opcode;
                Phase::Fetch
            }
            Target::Write(_, _) => unreachable!("reached decode phase after memory write"),
        }
    }

    /// Performs the 'execute' phase.
    #[inline]
    pub fn execute(&mut self, bus: &mut Bus) -> Phase {
        // Execute this instruction
        let ins = self.ins.clone();
        ins.execute(self);
        match self.target {
            Target::Write(addr, val) => {
                // It was a store instruction; store the result
                bus.pending_write.get_or_insert(vec![
                    State::Addr(addr),
                    State::Data(val),
                    State::Mem(true),
                    State::Write(true),
                ]);
                Phase::Wait
            }
            Target::Read(_, _) => Phase::Fetch,
            Target::Opcode | Target::Operand | Target::Operand2 => {
                // Fetch the next instruction
                self.target = Target::Opcode;
                Phase::Fetch
            }
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
            let addr = if let Target::Read(addr, _) = self.target {
                addr
            } else {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                addr
            };
            // Issue a memory read and await the result
            bus.pending_write.get_or_insert(vec![
                State::Addr(addr),
                State::Mem(true),
                State::Write(false),
            ]);
            Phase::Wait
        }
    }

    /// Loads the specified source register from the memory address in the specified
    /// target register.
    ///
    /// Invalid source or target registers make this a no-op.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn ld_indirect(&mut self) {
        if let Some((source, target)) = source_and_target_from(self.op() as u8) {
            self.read_mem(self.regs.get(source), target);
        }
    }

    /// Returns the 16-bit value of the two operand registers.
    #[inline]
    #[must_use]
    pub fn op(&self) -> u16 {
        u16::from_be_bytes([self.op_hi, self.op_lo])
    }

    /// Sets the next sequencer target to read `reg` from memory at `addr`.
    #[inline]
    pub fn read_mem(&mut self, addr: u16, reg: Reg) {
        self.target = Target::Read(addr, reg);
    }

    /// Resets the CPU to its power-on state.
    ///
    /// The initial state is: all registers and flags zero, PC zero, not halted, phase
    /// 'fetch' and target 'opcode'.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Stores the contents of the specified source register to the memory address in
    /// the specified target register.
    ///
    /// Invalid source or target registers make this a no-op.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn store_indirect(&mut self) {
        if let Some((source, target)) = source_and_target_from(self.op() as u8) {
            self.write_mem(self.regs.get(target), source);
        }
    }

    /// Performs the 'wait for memory' phase.
    #[inline]
    pub fn wait(&mut self) -> Phase {
        match self.target {
            Target::Write(_, _) => {
                // We were waiting for a memory write to complete; now fetch the next
                // instruction
                self.target = Target::Opcode;
                Phase::Fetch
            }
            Target::Opcode | Target::Operand | Target::Operand2 | Target::Read(_, _) => {
                // We were waiting for an opcode, operand, or memory read; now decode it
                Phase::Decode
            }
        }
    }

    /// Sets the next sequencer target to write `reg` to memory at `addr`.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn write_mem(&mut self, addr: u16, reg: Reg) {
        self.target = Target::Write(addr, self.regs.get(reg) as u8);
    }
}

/// The state of CPU's flags.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Flags {
    /// Indicates carry (from addition) or 'no borrow' (from subtraction or comparison).
    pub carry: bool,
    /// Indicates a zero result from the last operation.
    pub zero: bool,
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
    /// Memory read.
    Read(u16, Reg),
    /// Memory write.
    Write(u16, u8),
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use crate::{asm::asm, cpu::Phase::*, cpu::Target::*, regs::Reg::*, system::System};

    use super::*;

    #[test]
    fn cpu_states_are_correct_for_1_byte_instruction() {
        let mut sys = System::default();
        sys.mem
            .load(
                0x0000,
                &asm("
                    nop
                    halt"),
            )
            .unwrap();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Opcode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Execute);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Opcode);
        assert_eq!(sys.cpu.pc, 0x0001);
    }

    #[test]
    fn cpu_states_are_correct_for_2_byte_instruction() {
        let mut sys = System::default();
        sys.mem
            .load(
                0x0000,
                &asm("
                    ld a, 0xFF
                    halt"),
            )
            .unwrap();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Opcode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Operand);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Execute);
        sys.tick();
        assert_eq!(sys.cpu.regs.get(A), 0x00FF);
        assert_eq!(sys.cpu.pc, 0x0002);
    }

    #[test]
    fn cpu_states_are_correct_for_3_byte_instruction() {
        let mut sys = System::default();
        sys.mem
            .load(
                0x0000,
                &asm("
                    ld ab, 0xBEEF
                    halt"),
            )
            .unwrap();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Opcode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Operand);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Operand2);
        assert_eq!(sys.cpu.pc, 0x0002);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Execute);
        sys.tick();
        assert_eq!(sys.cpu.regs.get(AB), 0xBEEF);
        assert_eq!(sys.cpu.pc, 0x0003);
    }

    #[test]
    fn cpu_states_are_correct_for_mem_read_instruction() {
        let mut sys = System::default();
        sys.mem.set(0x0100, 0xFF);
        sys.cpu.regs.set(Reg::CD, 0x0100);
        sys.mem
            .load(
                0x0000,
                &asm("
                    ld b, (cd)
                    halt"),
            )
            .unwrap();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Opcode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Operand);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        assert_eq!(sys.cpu.target, Operand);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Execute);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Read(0x0100, Reg::B));
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        assert_eq!(sys.cpu.target, Read(0x0100, Reg::B));
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.regs.get(B), 0x00FF);
        assert_eq!(sys.cpu.pc, 0x0002);
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Opcode);
    }

    #[test]
    fn cpu_states_are_correct_for_mem_write_instruction() {
        let mut sys = System::default();
        sys.mem
            .load(
                0x0000,
                &asm("
                    ld 0xBEEF, a
                    halt"),
            )
            .unwrap();
        sys.cpu.regs.set(A, 0xFF);
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Opcode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Operand);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Fetch);
        assert_eq!(sys.cpu.target, Operand2);
        assert_eq!(sys.cpu.pc, 0x0002);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        sys.tick();
        assert_eq!(sys.cpu.phase, Decode);
        sys.tick();
        assert_eq!(sys.cpu.phase, Execute);
        sys.tick();
        assert_eq!(sys.cpu.phase, Wait);
        assert_eq!(sys.cpu.target, Write(0xBEEF, 0xFF));
        sys.tick();
        assert_eq!(sys.cpu.pc, 0x0003);
    }

    #[expect(clippy::bool_assert_comparison, reason = "clarity")]
    #[test]
    fn reset_resets_cpu() {
        let mut cpu = Cpu::default();
        cpu.regs.set(AB, 0xBEEF);
        cpu.pc = 0xC000;
        cpu.flags.carry = true;
        cpu.flags.zero = true;
        cpu.reset();
        assert_eq!(cpu.regs.get(AB), 0x0000, "AB not reset");
        assert_eq!(cpu.pc, 0x0000, "PC not reset");
        assert_eq!(cpu.flags.carry, false, "carry not reset");
        assert_eq!(cpu.flags.zero, false, "zero not reset");
    }
}
