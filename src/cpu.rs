use core::fmt::{Display, Formatter};

use crate::{
    instructions::{InstructionKind, Operands},
    regs::{Reg, Regs},
    system::{Bus, BusState, Device},
};

use State::*;

/// The system CPU.
#[non_exhaustive]
#[derive(Debug)]
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
    /// The CPU's registers.
    pub regs: Regs,
    /// The current state.
    pub state: State,
}

impl Default for Cpu {
    #[inline]
    fn default() -> Self {
        Self {
            flags: Flags::default(),
            halt: Default::default(),
            ins: InstructionKind::Nop,
            op_hi: Default::default(),
            op_lo: Default::default(),
            pc: Default::default(),
            state: State::default(),
            regs: Regs::default(),
        }
    }
}

impl Device for Cpu {
    /// Performs the current phase, and sets the next phase.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.state = match self.state {
            DecodeOpcode => {
                let opcode = bus.data;
                self.ins = InstructionKind::try_from(opcode).unwrap_or(InstructionKind::Nop);
                match self.ins.operands() {
                    Operands::Zero => {
                        // No operands needed, go straight to 'execute'
                        bus.pending_write.get_or_insert(vec![BusState::Mem(false)]);
                        Execute
                    }
                    Operands::One => {
                        self.fetch_and_advance(bus);
                        WaitOp1of1
                    }
                    Operands::Two => {
                        self.fetch_and_advance(bus);
                        WaitOp1of2
                    }
                }
            }
            Execute => {
                if self.halt {
                    // Just keep looping in this state
                    Execute
                } else {
                    let ins = self.ins;
                    ins.execute(self, bus);
                    self.state
                }
            }
            FetchOpcode | WaitStore1of1 => {
                self.fetch_and_advance(bus);
                WaitOpcode
            }
            ReadLoad1of1(reg) => {
                self.regs.set(reg, u16::from(bus.data));
                self.fetch_and_advance(bus);
                WaitOpcode
            }
            ReadOp1of1 => {
                self.op_lo = bus.data;
                Execute
            }
            ReadOp1of2 => {
                self.op_lo = bus.data;
                self.fetch_and_advance(bus);
                WaitOp2of2
            }
            ReadOp2of2 => {
                self.op_hi = bus.data;
                Execute
            }
            WaitLoad1of1(reg) => ReadLoad1of1(reg),
            WaitOp1of1 => ReadOp1of1,
            WaitOp1of2 => ReadOp1of2,
            WaitOp2of2 => ReadOp2of2,
            WaitOpcode => DecodeOpcode,
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

    /// Issues a memory fetch and advances PC.
    #[inline]
    pub fn fetch_and_advance(&mut self, bus: &mut Bus) {
        bus.pending_write.get_or_insert(vec![
            BusState::Addr(self.pc),
            BusState::Mem(true),
            BusState::Write(false),
        ]);
        self.pc = self.pc.wrapping_add(1);
    }

    /// Returns the 16-bit value of the two operand registers.
    #[inline]
    #[must_use]
    pub fn op(&self) -> u16 {
        u16::from_be_bytes([self.op_hi, self.op_lo])
    }

    /// Resets the CPU to its power-on state.
    ///
    /// The initial state is: all registers and flags zero, PC zero, not halted, phase
    /// 'fetch' and target 'opcode'.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
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

/// The state of the CPU on the next tick.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum State {
    /// Reads the opcode from the data bus.
    DecodeOpcode,
    /// Executes the current instruction.
    Execute,
    /// Requests the next opcode from memory.
    #[default]
    FetchOpcode,
    /// Loads a register from the bus.
    ReadLoad1of1(Reg),
    /// Reads a single operand from the bus.
    ReadOp1of1,
    /// Reads the first of two operands from the bus.
    ReadOp1of2,
    /// Reads the second of two operands from the bus.
    ReadOp2of2,
    /// Waits for a byte from memory to load a register.
    WaitLoad1of1(Reg),
    /// Waits for a single operand read from memory.
    WaitOp1of1,
    /// Waits for the first of two operands from memory.
    WaitOp1of2,
    /// Waits for the second of two operands from memory.
    WaitOp2of2,
    /// Waits for an opcode fetch to complete.
    WaitOpcode,
    /// Waits for a byte to be written to memory.
    WaitStore1of1,
}

impl Display for State {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                DecodeOpcode => "DCOD",
                Execute => "EXEC",
                FetchOpcode => "FOPC",
                ReadLoad1of1(_) => "RL11",
                ReadOp1of1 => "RO11",
                ReadOp1of2 => "RO12",
                ReadOp2of2 => "RO22",
                WaitLoad1of1(_) => "WL11",
                WaitOp1of1 => "WO11",
                WaitOp1of2 => "WO12",
                WaitOp2of2 => "WO22",
                WaitOpcode => "WOPC",
                WaitStore1of1 => "WS11",
            }
        )
    }
}

// #[expect(clippy::exhaustive_enums, reason = "this actually is exhaustive")]
// /// The target of the next fetch or wait phase.
// #[derive(Debug, Default, PartialEq)]
// pub enum Target {
//     /// An opcode.
//     #[default]
//     Opcode,
//     /// A 1-byte operand (or low byte of a 2-byte operand).
//     Operand,
//     /// A second 1-byte operand (or high byte of a 2-byte operand).
//     Operand2,
//     /// Memory read.
//     Read(u16, Reg),
//     /// Memory write.
//     Write(u16, u8),
// }

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use crate::{asm::asm, regs::Reg::*, system::System};

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
        assert_eq!(sys.cpu.state, FetchOpcode);
        assert_eq!(sys.cpu.pc, 0x0000);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.state, DecodeOpcode);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0002);
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
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.pc, 0x0001);
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, DecodeOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp1of1);
        assert_eq!(sys.cpu.pc, 0x0002);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp1of1);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.regs.get(A), 0x00FF);
        assert_eq!(sys.cpu.pc, 0x0003);
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
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, DecodeOpcode);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp1of2);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp1of2);
        assert_eq!(sys.cpu.pc, 0x0002);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp2of2);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp2of2);
        assert_eq!(sys.cpu.pc, 0x0003);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.regs.get(AB), 0xBEEF);
        assert_eq!(sys.cpu.pc, 0x0004);
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
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0001);
        sys.tick();
        assert_eq!(sys.cpu.state, DecodeOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp1of1);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp1of1);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.pc, 0x0002);
        assert_eq!(sys.cpu.state, WaitLoad1of1(B));
        sys.tick();
        assert_eq!(sys.cpu.state, ReadLoad1of1(B));
        sys.tick();
        assert_eq!(sys.cpu.regs.get(B), 0x00FF);
        assert_eq!(sys.cpu.pc, 0x0003);
        assert_eq!(sys.cpu.state, WaitOpcode);
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
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, DecodeOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp1of2);
        assert_eq!(sys.cpu.pc, 0x0002);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp1of2);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp2of2);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp2of2);
        assert_eq!(sys.cpu.pc, 0x0003);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitStore1of1);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0004);
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
