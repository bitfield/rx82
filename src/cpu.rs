use core::fmt::{Display, Formatter};

use crate::{
    bus::Bus,
    instructions::{InstructionKind, Operands},
    regs::{Reg, Regs, source_and_target_from},
    system::Device,
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
    /// Transitions to the next state.
    #[inline]
    fn tick(&mut self, bus: &mut Bus) {
        self.state = match self.state {
            Decode => {
                let opcode = bus.data;
                self.ins = InstructionKind::try_from(opcode).unwrap_or(InstructionKind::Nop);
                match self.ins.operands() {
                    Operands::Zero => {
                        // No operands needed, go straight to 'execute'
                        bus.disable_mem();
                        Execute
                    }
                    Operands::One => {
                        self.fetch_and_advance(bus);
                        WaitOp
                    }
                    Operands::Two => {
                        self.fetch_and_advance(bus);
                        WaitOp1of2
                    }
                }
            }
            Execute => {
                let ins = self.ins;
                // default next state, but may be overridden by instruction
                self.state = FetchOpcode;
                ins.execute(self, bus);
                self.state
            }
            FetchOpcode => {
                if self.halt {
                    FetchOpcode
                } else {
                    self.fetch_and_advance(bus);
                    WaitOpcode
                }
            }
            ReadDec(addr) => {
                let mut val = bus.data;
                val = val.wrapping_sub(1);
                bus.mem_write(addr, val);
                self.flags.zero = val == 0;
                FetchOpcode
            }
            ReadInc(addr) => {
                let mut val = bus.data;
                val = val.wrapping_add(1);
                bus.mem_write(addr, val);
                self.flags.zero = val == 0;
                FetchOpcode
            }
            ReadLoad(reg) => {
                self.regs.set(reg, u16::from(bus.data));
                FetchOpcode
            }
            ReadOp => {
                self.op_hi = 0;
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
            WaitDec(addr) => ReadDec(addr),
            WaitInc(addr) => ReadInc(addr),
            WaitLoad(reg) => ReadLoad(reg),
            WaitOp => ReadOp,
            WaitOp1of2 => ReadOp1of2,
            WaitOp2of2 => ReadOp2of2,
            WaitOpcode => Decode,
        };
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

    /// Compares the value in register `reg` with the operand, updating flags.
    #[inline]
    pub fn cmp(&mut self, reg: Reg, rhs: u16) {
        let lhs = self.regs.get(reg);
        self.flags.zero = lhs == rhs;
        self.flags.carry = lhs >= rhs;
    }

    /// Decrements the value at address in `reg`, updating flags.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn dec_indirect(&mut self, bus: &mut Bus) {
        if let Some((source, _)) = source_and_target_from(self.op() as u8)
            && source.is16()
        {
            let addr = self.regs.get(source);
            self.dec_mem(addr, bus);
        } else {
            self.illegal();
        }
    }

    /// Decrements the value in address `addr`, updating flags.
    #[inline]
    pub fn dec_mem(&mut self, addr: u16, bus: &mut Bus) {
        bus.mem_read(addr);
        self.state = WaitDec(addr);
    }

    /// Decrements the value in register `reg`, updating flags.
    #[inline]
    pub fn decrement(&mut self, reg: Reg) {
        let value = self.regs.get(reg).wrapping_sub(1);
        self.flags.zero = self.regs.set(reg, value) == 0;
    }

    /// Issues a memory fetch and advances PC.
    #[inline]
    pub fn fetch_and_advance(&mut self, bus: &mut Bus) {
        bus.mem_read(self.pc);
        self.pc = self.pc.wrapping_add(1);
    }

    /// Halts the CPU.
    #[inline]
    pub fn halt(&mut self) {
        self.halt = true;
    }

    /// Raises an illegal instruction exception.
    #[inline]
    pub fn illegal(&mut self) {
        self.halt = true;
    }

    /// Increments the value at address in `reg`, updating flags.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn inc_indirect(&mut self, bus: &mut Bus) {
        if let Some((source, _)) = source_and_target_from(self.op() as u8)
            && source.is16()
        {
            let addr = self.regs.get(source);
            self.inc_mem(addr, bus);
        } else {
            self.illegal();
        }
    }

    /// Increments the value in address `addr`, updating flags.
    #[inline]
    pub fn inc_mem(&mut self, addr: u16, bus: &mut Bus) {
        bus.mem_read(addr);
        self.state = WaitInc(addr);
    }

    /// Increments the value in register `reg`, updating flags.
    #[inline]
    pub fn increment(&mut self, reg: Reg) {
        let value = self.regs.get(reg).wrapping_add(1);
        self.flags.zero = self.regs.set(reg, value) == 0;
    }

    /// Executes a load register indirect instruction.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn ld_reg_indirect(&mut self, bus: &mut Bus) {
        if let Some((source, target)) = source_and_target_from(self.op() as u8) {
            bus.mem_read(self.regs.get(source));
            self.state = WaitLoad(target);
        } else {
            self.illegal();
        }
    }

    /// Returns the 16-bit value of the two operand registers.
    #[inline]
    #[must_use]
    pub fn op(&self) -> u16 {
        u16::from_be_bytes([self.op_hi, self.op_lo])
    }

    /// Resets the CPU to its power-on state.
    ///
    /// The initial state is: all registers and flags zero, PC zero, not halted, state
    /// [`FetchOpcode`](State::FetchOpcode).
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Executes a store register direct instruction.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn store_reg_direct(&mut self, reg: Reg, bus: &mut Bus) {
        bus.mem_write(self.op(), self.regs.get(reg) as u8);
        self.state = FetchOpcode;
    }

    /// Executes a store register indirect instruction.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn store_reg_indirect(&mut self, bus: &mut Bus) {
        if let Some((source, target)) = source_and_target_from(self.op() as u8) {
            bus.mem_write(self.regs.get(target), self.regs.get(source) as u8);
            self.state = FetchOpcode;
        } else {
            self.illegal();
        }
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
    Decode,
    /// Executes the current instruction.
    Execute,
    /// Requests the next opcode from memory.
    #[default]
    FetchOpcode,
    /// Reads a byte from memory for a decrement instruction.
    ReadDec(u16),
    /// Reads a byte from memory for an increment instruction.
    ReadInc(u16),
    /// Loads a register from the bus.
    ReadLoad(Reg),
    /// Reads a single operand from the bus.
    ReadOp,
    /// Reads the first of two operands from the bus.
    ReadOp1of2,
    /// Reads the second of two operands from the bus.
    ReadOp2of2,
    /// Waits for a byte from memory for a decrement instruction.
    WaitDec(u16),
    /// Waits for a byte from memory for an increment instruction.
    WaitInc(u16),
    /// Waits for a byte from memory to load a register.
    WaitLoad(Reg),
    /// Waits for a single operand read from memory.
    WaitOp,
    /// Waits for the first of two operands from memory.
    WaitOp1of2,
    /// Waits for the second of two operands from memory.
    WaitOp2of2,
    /// Waits for an opcode fetch to complete.
    WaitOpcode,
}

impl Display for State {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Decode => "DCOD",
                Execute => "EXEC",
                FetchOpcode => "FOPC",
                ReadLoad(_) => "RDLD",
                ReadOp => "RDOP",
                ReadOp1of2 => "RO12",
                ReadOp2of2 => "RO22",
                WaitLoad(_) => "WTLD",
                WaitOp => "WTOP",
                WaitOp1of2 => "WO12",
                WaitOp2of2 => "WO22",
                WaitOpcode => "WOPC",
                ReadDec(_) => "RDEC",
                ReadInc(_) => "RINC",
                WaitDec(_) => "WDEC",
                WaitInc(_) => "WINC",
            }
        )
    }
}

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
                0x0100,
                &asm("
                    nop
                    halt"),
            )
            .unwrap();
        sys.cpu.pc = 0x0100;
        assert_eq!(sys.cpu.state, FetchOpcode);
        assert_eq!(sys.cpu.pc, 0x0100);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0101);
        sys.tick();
        assert_eq!(sys.cpu.state, Decode);
        assert_eq!(sys.cpu.pc, 0x0101);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        assert_eq!(sys.cpu.pc, 0x0101);
        sys.tick();
        assert_eq!(sys.cpu.state, FetchOpcode);
        assert_eq!(sys.cpu.pc, 0x0101);
    }

    #[test]
    fn cpu_states_are_correct_for_2_byte_instruction() {
        let mut sys = System::default();
        sys.mem
            .load(
                0x0100,
                &asm("
                    ld a, 0xFF
                    halt"),
            )
            .unwrap();
        sys.cpu.pc = 0x0100;
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.pc, 0x0101);
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, Decode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp);
        assert_eq!(sys.cpu.pc, 0x0102);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.regs.get(A), 0x00FF);
        assert_eq!(sys.cpu.pc, 0x0102);
    }

    #[test]
    fn cpu_states_are_correct_for_3_byte_instruction() {
        let mut sys = System::default();
        sys.mem
            .load(
                0x0100,
                &asm("
                    ld ab, 0xBEEF
                    halt"),
            )
            .unwrap();
        sys.cpu.pc = 0x0100;
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, Decode);
        assert_eq!(sys.cpu.pc, 0x0101);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp1of2);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp1of2);
        assert_eq!(sys.cpu.pc, 0x0102);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp2of2);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp2of2);
        assert_eq!(sys.cpu.pc, 0x0103);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.regs.get(AB), 0xBEEF);
        assert_eq!(sys.cpu.pc, 0x0103);
    }

    #[test]
    fn cpu_states_are_correct_for_mem_read_instruction() {
        let mut sys = System::default();
        sys.mem.set(0x0110, 0xFF);
        sys.cpu.regs.set(Reg::CD, 0x0110);
        sys.mem
            .load(
                0x0100,
                &asm("
                    ld b, (cd)
                    halt"),
            )
            .unwrap();
        sys.cpu.pc = 0x0100;
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0101);
        sys.tick();
        assert_eq!(sys.cpu.state, Decode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.pc, 0x0102);
        assert_eq!(sys.cpu.state, WaitLoad(B));
        sys.tick();
        assert_eq!(sys.cpu.state, ReadLoad(B));
        sys.tick();
        assert_eq!(sys.cpu.regs.get(B), 0x00FF);
        assert_eq!(sys.cpu.pc, 0x0102);
        assert_eq!(sys.cpu.state, FetchOpcode);
    }

    #[test]
    fn cpu_states_are_correct_for_mem_write_instruction() {
        let mut sys = System::default();
        sys.mem
            .load(
                0x0100,
                &asm("
                    ld 0xBEEF, a
                    halt"),
            )
            .unwrap();
        sys.cpu.pc = 0x0100;
        sys.cpu.regs.set(A, 0xFF);
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, Decode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp1of2);
        assert_eq!(sys.cpu.pc, 0x0102);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp1of2);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOp2of2);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOp2of2);
        assert_eq!(sys.cpu.pc, 0x0103);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0104);
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
