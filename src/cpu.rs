use core::fmt::{Display, Formatter};

use crate::{
    bus::Bus,
    instructions::{InstructionKind, Operands},
    regs::{Reg, Regs, source_and_target_from},
    system::Device,
};

use State::*;

pub const VEC_RESET: u16 = 0xFFFE;

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
    #[expect(clippy::too_many_lines, reason = "it's just long")]
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
                        WaitOpLo
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
                self.op_lo = bus.data;
                self.regs.set(reg, self.op());
                FetchOpcode
            }
            ReadOp => {
                self.op_hi = 0;
                self.op_lo = bus.data;
                Execute
            }
            ReadOpLo => {
                self.op_lo = bus.data;
                self.fetch_and_advance(bus);
                WaitOpHi
            }
            ReadOpHi => {
                self.op_hi = bus.data;
                Execute
            }
            ReadResetLo => {
                self.op_lo = bus.data;
                bus.mem_read(VEC_RESET.wrapping_add(1));
                WaitAddrHi
            }
            ReadAddrHi => {
                self.op_hi = bus.data;
                self.pc = self.op();
                FetchOpcode
            }
            ReadRetHi => {
                self.op_hi = bus.data;
                self.stack_pop(bus);
                WaitRetLo
            }
            ReadRetLo => {
                self.op_lo = bus.data;
                self.pc = self.op();
                FetchOpcode
            }
            ReadStackHi(reg) => {
                self.op_hi = bus.data;
                self.stack_pop(bus);
                WaitLoad(reg)
            }
            ReadTrapVecLo(addr) => {
                self.op_lo = bus.data;
                bus.mem_read(addr);
                WaitAddrHi
            }
            WaitCall(hi, subr_addr) => {
                self.stack_push(hi, bus);
                self.pc = subr_addr;
                FetchOpcode
            }
            WaitDec(addr) => ReadDec(addr),
            WaitInc(addr) => ReadInc(addr),
            WaitLoad(reg) => ReadLoad(reg),
            WaitOp => ReadOp,
            WaitOpLo => ReadOpLo,
            WaitOpHi => ReadOpHi,
            WaitOpcode => Decode,
            WaitResetLo => ReadResetLo,
            WaitAddrHi => ReadAddrHi,
            WaitStackHi(reg) => ReadStackHi(reg),
            WaitPush(val) => {
                self.stack_push(val, bus);
                FetchOpcode
            }
            WaitRetHi => ReadRetHi,
            WaitRetLo => ReadRetLo,
            WaitTrapCode(trap_code) => {
                let mut vec_addr = u16::from(trap_code.strict_mul(2));
                bus.mem_read(vec_addr);
                vec_addr = vec_addr.wrapping_add(1);
                WaitTrapVecLo(vec_addr)
            }
            WaitTrapLo(hi, trap_code) => {
                self.stack_push(hi, bus);
                WaitTrapHi(trap_code)
            }
            WaitTrapHi(trap_code) => {
                self.stack_push(trap_code, bus);
                WaitTrapCode(trap_code)
            }
            WaitTrapVecLo(addr) => ReadTrapVecLo(addr),
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

    /// Calls the subroutine at `addr`, pushing the return address on the stack.
    #[inline]
    pub fn call(&mut self, subr_addr: u16, bus: &mut Bus) {
        let ret_addr = self.pc;
        let [hi, lo] = ret_addr.to_be_bytes();
        self.stack_push(lo, bus);
        self.state = WaitCall(hi, subr_addr);
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

    /// Executes a load register register instruction.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn ld_reg_reg(&mut self) {
        if let Some((source, target)) = source_and_target_from(self.op() as u8)
            && source.is16() == target.is16()
        {
            self.regs.set(target, self.regs.get(source));
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

    /// Executes a pop instruction with `reg`.
    #[inline]
    pub fn pop(&mut self, reg: Reg, bus: &mut Bus) {
        self.stack_pop(bus);
        if reg.is16() {
            self.state = WaitStackHi(reg);
        } else {
            self.op_hi = 0;
            self.state = WaitLoad(reg);
        }
    }

    /// Executes a push instruction with `reg`.
    #[expect(clippy::cast_possible_truncation, reason = "truncation is correct")]
    #[inline]
    pub fn push(&mut self, reg: Reg, bus: &mut Bus) {
        let val = self.regs.get(reg);
        if reg.is16() {
            let [hi, lo] = val.to_be_bytes();
            self.stack_push(lo, bus);
            self.state = WaitPush(hi);
        } else {
            self.stack_push(val as u8, bus);
            self.state = FetchOpcode;
        }
    }

    /// Resets the CPU to its power-on state.
    ///
    /// The initial state is: all registers and flags zero, not halted, state
    /// [`FetchOpcode`](State::FetchOpcode), and PC = the reset vector.
    #[inline]
    pub fn reset(&mut self, bus: &mut Bus) {
        *self = Self::default();
        bus.mem_read(VEC_RESET);
        self.state = WaitResetLo;
    }

    /// Returns from a subroutine to a return address on the stack.
    #[inline]
    pub fn ret(&mut self, bus: &mut Bus) {
        self.stack_pop(bus);
        self.state = WaitRetHi;
    }

    /// Returns from a trap to a return address on the stack.
    #[inline]
    pub fn rti(&mut self, bus: &mut Bus) {
        let mut addr = self.regs.get(Reg::SP);
        addr = addr.wrapping_add(2); // skip trap code
        bus.mem_read(addr);
        self.regs.set(Reg::SP, addr);
        self.state = WaitRetHi;
    }

    /// Reads the current top-of-stack value, adjusting SP.
    #[inline]
    pub fn stack_pop(&mut self, bus: &mut Bus) {
        let mut addr = self.regs.get(Reg::SP);
        addr = addr.wrapping_add(1);
        bus.mem_read(addr);
        self.regs.set(Reg::SP, addr);
    }

    /// Writes `val` to the stack, adjusting SP.
    #[inline]
    pub fn stack_push(&mut self, val: u8, bus: &mut Bus) {
        let mut addr = self.regs.get(Reg::SP);
        bus.mem_write(addr, val);
        addr = addr.wrapping_sub(1);
        self.regs.set(Reg::SP, addr);
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

    /// Executes a trap instruction.
    ///
    /// The trap code is used to select a vector from the trap table, and the CPU jumps
    /// to that address after pushing the return address and the trap code to the stack.
    /// Calls the subroutine at `addr`, pushing the return address on the stack.
    #[inline]
    pub fn trap(&mut self, trap_code: u8, bus: &mut Bus) {
        if trap_code >= 0x40 {
            self.illegal();
            return;
        }
        let ret_addr = self.pc;
        let [hi, lo] = ret_addr.to_be_bytes();
        self.stack_push(lo, bus);
        self.state = WaitTrapLo(hi, trap_code);
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
    /// Reads the high byte of the address to jump to.
    ReadAddrHi,
    /// Reads a byte from memory for a decrement instruction.
    ReadDec(u16),
    /// Reads a byte from memory for an increment instruction.
    ReadInc(u16),
    /// Loads a register from the bus.
    ReadLoad(Reg),
    /// Reads a single operand from the bus.
    ReadOp,
    /// Reads the second of two operands from the bus.
    ReadOpHi,
    /// Reads the first of two operands from the bus.
    ReadOpLo,
    /// Reads the low byte of the reset vector from the bus.
    ReadResetLo,
    /// Reads the high byte of the return address for a ret instruction.
    ReadRetHi,
    /// Reads the low byte of the return address for a ret instruction.
    ReadRetLo,
    /// Reads the first of two stack values from the bus.
    ReadStackHi(Reg),
    /// Reads the low byte of the trap vector for a trap instruction.
    ReadTrapVecLo(u16),
    /// Waits for the high byte of the address to jump to.
    WaitAddrHi,
    /// Waits for the low byte of the return address to be pushed for a call
    /// instruction.
    WaitCall(u8, u16),
    /// Waits for a byte from memory for a decrement instruction.
    WaitDec(u16),
    /// Waits for a byte from memory for an increment instruction.
    WaitInc(u16),
    /// Waits for a byte from memory to load a register.
    WaitLoad(Reg),
    /// Waits for a single operand read from memory.
    WaitOp,
    /// Waits for the second of two operands from memory.
    WaitOpHi,
    /// Waits for the first of two operands from memory.
    WaitOpLo,
    /// Waits for an opcode fetch to complete.
    WaitOpcode,
    /// Waits for a stack push, before pushing another value.
    WaitPush(u8),
    /// Waits for the low byte of the reset vector.
    WaitResetLo,
    /// Waits for the high byte of the return address for a ret instruction.
    WaitRetHi,
    /// Waits for the low byte of the return address for a ret instruction.
    WaitRetLo,
    /// Waits for the first of 2 stack pops to a register.
    WaitStackHi(Reg),
    /// Waits for the trap code to be pushed for a trap instruction.
    WaitTrapCode(u8),
    /// Waits for the high byte of the return address to be pushed for a trap
    /// instruction.
    WaitTrapHi(u8),
    /// Waits for the low byte of the return address to be pushed for a trap
    /// instruction.
    WaitTrapLo(u8, u8),
    /// Waits for the low byte of the trap vector for a trap instruction.
    WaitTrapVecLo(u16),
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
                ReadAddrHi => "RDAH",
                ReadDec(_) => "RDEC",
                ReadInc(_) => "RINC",
                ReadLoad(_) => "RDLD",
                ReadOp => "RDOP",
                ReadOpHi => "ROPH",
                ReadOpLo => "ROPL",
                ReadResetLo => "RRSL",
                ReadRetHi => "RRTH",
                ReadRetLo => "RRTL",
                ReadStackHi(_) => "RSTH",
                ReadTrapVecLo(_) => "RTVL",
                WaitCall(_, _) => "WCAL",
                WaitDec(_) => "WDEC",
                WaitInc(_) => "WINC",
                WaitLoad(_) => "WTLD",
                WaitOp => "WTOP",
                WaitOpHi => "WOPH",
                WaitOpLo => "WOPL",
                WaitOpcode => "WOPC",
                WaitPush(_) => "WWRT",
                WaitAddrHi => "WTAH",
                WaitResetLo => "WRSL",
                WaitRetHi => "WRTH",
                WaitRetLo => "WRTL",
                WaitStackHi(_) => "WSTH",
                WaitTrapCode(_) => "WTTC",
                WaitTrapHi(_) => "WTTH",
                WaitTrapLo(_, _) => "WTTL",
                WaitTrapVecLo(_) => "WTVL",
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
        assert_eq!(sys.cpu.state, WaitOpLo);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOpLo);
        assert_eq!(sys.cpu.pc, 0x0102);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpHi);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOpHi);
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
        assert_eq!(sys.cpu.state, WaitOpLo);
        assert_eq!(sys.cpu.pc, 0x0102);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOpLo);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpHi);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadOpHi);
        assert_eq!(sys.cpu.pc, 0x0103);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        assert_eq!(sys.cpu.pc, 0x0104);
    }

    #[test]
    fn cpu_states_are_correct_for_16_bit_pop_instruction() {
        let mut sys = System::default();
        sys.mem.load(0xBFFE, &[0xBA, 0xBE]).unwrap();
        sys.cpu.regs.set(SP, 0xBFFD);
        sys.mem
            .load(
                0x0100,
                &asm("
                    pop cd
                    halt"),
            )
            .unwrap();
        sys.cpu.pc = 0x0100;
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, Decode);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitStackHi(CD));
        sys.tick();
        assert_eq!(sys.cpu.state, ReadStackHi(CD));
        sys.tick();
        assert_eq!(sys.cpu.state, WaitLoad(CD));
        sys.tick();
        assert_eq!(sys.cpu.state, ReadLoad(CD));
        sys.tick();
        assert_eq!(sys.cpu.state, FetchOpcode);
    }

    #[test]
    fn cpu_states_are_correct_for_16_bit_push_instruction() {
        let mut sys = System::default();
        sys.cpu.regs.set(SP, 0xBFFF);
        sys.cpu.regs.set(AB, 0xCAFE);
        sys.mem
            .load(
                0x0100,
                &asm("
                    push ab
                    halt"),
            )
            .unwrap();
        sys.cpu.pc = 0x0100;
        assert_eq!(sys.cpu.state, FetchOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitOpcode);
        sys.tick();
        assert_eq!(sys.cpu.state, Decode);
        sys.tick();
        assert_eq!(sys.cpu.state, Execute);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitPush(0xCA));
        sys.tick();
        assert_eq!(sys.cpu.state, FetchOpcode);
    }

    #[expect(clippy::bool_assert_comparison, reason = "clarity")]
    #[test]
    fn reset_resets_cpu() {
        let mut sys = System::default();
        sys.cpu.regs.set(AB, 0xBEEF);
        sys.cpu.regs.set(SP, 0xFFFD);
        sys.cpu.pc = 0x0000;
        sys.cpu.flags.carry = true;
        sys.cpu.flags.zero = true;
        sys.cpu.reset(&mut sys.bus);
        assert_eq!(sys.cpu.state, WaitResetLo);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadResetLo);
        sys.tick();
        assert_eq!(sys.cpu.state, WaitAddrHi);
        sys.tick();
        assert_eq!(sys.cpu.state, ReadAddrHi);
        sys.tick();
        assert_eq!(sys.cpu.state, FetchOpcode);
        assert_eq!(sys.cpu.regs.get(AB), 0x0000, "AB not reset");
        assert_eq!(sys.cpu.regs.get(SP), 0x0000, "SP not reset");
        assert_eq!(sys.cpu.pc, 0xC000, "PC not initialized from reset vector");
        assert_eq!(sys.cpu.flags.carry, false, "carry not reset");
        assert_eq!(sys.cpu.flags.zero, false, "zero not reset");
    }
}
