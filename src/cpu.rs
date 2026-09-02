use core::fmt::{Display, Formatter};

use crate::{
    bus::Bus,
    instructions::{InstructionKind, Operands},
    regs::{Reg, Regs, source_and_target_from, source_from},
    system::Device,
};

use State::*;

/// Trap code for the 'illegal instruction' trap.
pub const TRAP_ILLEGAL: u8 = 0x00;

/// The hard-wired reset vector address.
pub const VEC_RESET: u16 = 0xFFFE;

/// The system CPU.
#[derive(Debug)]
pub struct Cpu {
    /// Flags.
    pub flags: Flags,
    /// Is the CPU halted?
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
    fn tick(&mut self, bus: &mut Bus) {
        self.state = match self.state {
            Decode => {
                let opcode = bus.data;
                let Ok(ins) = InstructionKind::try_from(opcode) else {
                    self.trap(TRAP_ILLEGAL, bus);
                    return;
                };
                self.ins = ins;
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
                bus.write_mem(addr, val);
                self.flags.zero = val == 0;
                FetchOpcode
            }
            ReadInc(addr) => {
                let mut val = bus.data;
                val = val.wrapping_add(1);
                bus.write_mem(addr, val);
                self.flags.zero = val == 0;
                FetchOpcode
            }
            ReadLoad(reg) => {
                self.op_lo = bus.data;
                if reg.is16() {
                    self.regs.set16(reg, self.op());
                } else {
                    self.regs.set(reg, self.op_lo);
                }
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
                bus.read_mem(VEC_RESET.wrapping_add(1));
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
                bus.read_mem(addr);
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
                bus.read_mem(vec_addr);
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
    /// Add with carry.
    pub fn add(&mut self, reg: Reg, addend: u8) {
        let augend = self.regs.get(reg);
        let (result1, carry1) = augend.overflowing_add(addend);
        let carry_in = u8::from(self.flags.carry);
        let (result2, carry2) = result1.overflowing_add(carry_in);
        self.flags.carry = carry1 || carry2;
        self.regs.set(reg, result2);
        self.flags.zero = result2 == 0;
    }

    /// Bitwise AND.
    pub fn and(&mut self, reg: Reg, mask: u8) {
        let value = self.regs.get(reg);
        let result = value & mask;
        self.regs.set(reg, result);
        self.flags.zero = result == 0;
    }

    /// Branches to PC+`dis`.
    #[expect(clippy::cast_possible_wrap, reason = "i8 to u16 is sound")]
    #[expect(clippy::cast_sign_loss, reason = "okay with wrapping_add")]
    pub fn branch(&mut self, dis: u8) {
        self.pc = self.pc.wrapping_add(dis as i8 as u16); // sign-extend displacement
    }

    /// Calls the subroutine at `addr`, pushing the return address on the stack.
    pub fn call(&mut self, addr: u16, bus: &mut Bus) {
        let ret_addr = self.pc;
        let [hi, lo] = ret_addr.to_be_bytes();
        self.stack_push(lo, bus);
        self.state = WaitCall(hi, addr);
    }

    /// Compares the value in register `reg` with the operand, updating flags.
    pub fn cmp(&mut self, reg: Reg, rhs: u8) {
        let lhs = self.regs.get(reg);
        self.flags.zero = lhs == rhs;
        self.flags.carry = lhs >= rhs;
    }

    /// Compares the value in register pair `reg` with the operand, updating flags.
    pub fn cmp16(&mut self, reg: Reg, rhs: u16) {
        let lhs = self.regs.get16(reg);
        self.flags.zero = lhs == rhs;
        self.flags.carry = lhs >= rhs;
    }

    /// Decrements the value in register `reg`, updating flags.
    pub fn dec(&mut self, reg: Reg) {
        let result = self.regs.get(reg).wrapping_sub(1);
        self.regs.set(reg, result);
        self.flags.zero = result == 0;
    }

    /// Decrements the value in register pair `reg`, updating flags.
    pub fn dec16(&mut self, reg: Reg) {
        let result = self.regs.get16(reg).wrapping_sub(1);
        self.regs.set16(reg, result);
        self.flags.zero = result == 0;
    }

    /// Decrements the value at the address in `reg`, updating flags.
    pub fn dec_indirect(&mut self, bus: &mut Bus) {
        if let Some(source) = source_from(self.op_lo)
            && source.is16()
        {
            let addr = self.regs.get16(source);
            self.dec_mem(addr, bus);
        } else {
            self.trap(TRAP_ILLEGAL, bus);
        }
    }

    /// Decrements the value at the address `addr`, updating flags.
    pub fn dec_mem(&mut self, addr: u16, bus: &mut Bus) {
        bus.read_mem(addr);
        self.state = WaitDec(addr);
    }

    /// Issues a memory fetch and advances PC.
    pub fn fetch_and_advance(&mut self, bus: &mut Bus) {
        bus.read_mem(self.pc);
        self.pc = self.pc.wrapping_add(1);
    }

    /// Halts the CPU.
    pub fn halt(&mut self) {
        self.halt = true;
    }

    /// Increments the value in register `reg`, updating flags.
    pub fn inc(&mut self, reg: Reg) {
        let result = self.regs.get(reg).wrapping_add(1);
        self.regs.set(reg, result);
        self.flags.zero = result == 0;
    }

    /// Increments the value in register pair `reg`, updating flags.
    pub fn inc16(&mut self, reg: Reg) {
        let result = self.regs.get16(reg).wrapping_add(1);
        self.regs.set16(reg, result);
        self.flags.zero = result == 0;
    }

    /// Increments the value at the address in `reg`, updating flags.
    pub fn inc_indirect(&mut self, bus: &mut Bus) {
        if let Some(source) = source_from(self.op_lo)
            && source.is16()
        {
            let addr = self.regs.get16(source);
            self.inc_mem(addr, bus);
        } else {
            self.trap(TRAP_ILLEGAL, bus);
        }
    }

    /// Increments the value at the address `addr`, updating flags.
    pub fn inc_mem(&mut self, addr: u16, bus: &mut Bus) {
        bus.read_mem(addr);
        self.state = WaitInc(addr);
    }

    /// Jumps to address `addr`.
    pub fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    /// Executes a load register indirect instruction.
    pub fn ld_reg_indirect(&mut self, bus: &mut Bus) {
        if let Some((source, target)) = source_and_target_from(self.op_lo) {
            bus.read_mem(self.regs.get16(source));
            self.state = WaitLoad(target);
        } else {
            self.trap(TRAP_ILLEGAL, bus);
        }
    }

    /// Executes a load register register instruction.
    pub fn ld_reg_reg(&mut self, bus: &mut Bus) {
        match source_and_target_from(self.op_lo) {
            Some((source, target)) if source.is16() && target.is16() => {
                self.regs.set16(target, self.regs.get16(source));
            }
            Some((source, target)) if !source.is16() && !target.is16() => {
                self.regs.set(target, self.regs.get(source));
            }
            _ => self.trap(TRAP_ILLEGAL, bus),
        }
    }

    /// Executes a logical shift right instruction.
    pub fn lsr(&mut self, reg: Reg, mut bits: u8) {
        bits = bits.clamp(1, 8);
        let mut value = self
            .regs
            .get(reg)
            .unbounded_shr(u32::from(bits.strict_sub(1))); // clamped >= 1
        let last_bit = value & 1;
        value = value.unbounded_shr(1);
        self.regs.set(reg, value);
        self.flags.carry = last_bit == 1;
    }

    /// Returns the 16-bit value of the two operand registers.
    #[must_use]
    pub fn op(&self) -> u16 {
        u16::from_be_bytes([self.op_hi, self.op_lo])
    }

    /// Executes a `pop` instruction with `reg`.
    pub fn pop(&mut self, reg: Reg, bus: &mut Bus) {
        self.stack_pop(bus);
        if reg.is16() {
            self.state = WaitStackHi(reg);
        } else {
            self.op_hi = 0;
            self.state = WaitLoad(reg);
        }
    }

    /// Executes a `push` instruction with `reg`.
    pub fn push(&mut self, reg: Reg, bus: &mut Bus) {
        if reg.is16() {
            let val = self.regs.get16(reg);
            let [hi, lo] = val.to_be_bytes();
            self.stack_push(lo, bus);
            self.state = WaitPush(hi);
        } else {
            let val = self.regs.get(reg);
            self.stack_push(val, bus);
        }
    }

    /// Resets the CPU to its power-on state.
    ///
    /// The initial state is: all registers and flags zero, not halted, state
    /// [`WaitResetLo`]. On the next tick, the CPU will request the low byte of the
    /// reset vector from the address [`VEC_RESET`].
    pub fn reset(&mut self, bus: &mut Bus) {
        *self = Self::default();
        bus.read_mem(VEC_RESET);
        self.state = WaitResetLo;
    }

    /// Returns from a subroutine to a return address on the stack.
    pub fn ret(&mut self, bus: &mut Bus) {
        self.stack_pop(bus);
        self.state = WaitRetHi;
    }

    /// Returns from a trap to a return address on the stack.
    pub fn rti(&mut self, bus: &mut Bus) {
        let mut addr = self.regs.get16(Reg::SP);
        addr = addr.wrapping_add(2); // skip trap code
        bus.read_mem(addr);
        self.regs.set16(Reg::SP, addr);
        self.state = WaitRetHi;
    }

    /// Reads the current top-of-stack value, adjusting SP.
    pub fn stack_pop(&mut self, bus: &mut Bus) {
        let mut addr = self.regs.get16(Reg::SP);
        addr = addr.wrapping_add(1);
        bus.read_mem(addr);
        self.regs.set16(Reg::SP, addr);
    }

    /// Writes `val` to the stack, adjusting SP.
    pub fn stack_push(&mut self, val: u8, bus: &mut Bus) {
        let mut addr = self.regs.get16(Reg::SP);
        bus.write_mem(addr, val);
        addr = addr.wrapping_sub(1);
        self.regs.set16(Reg::SP, addr);
    }

    /// Executes a store register direct instruction.
    pub fn store_reg_direct(&mut self, reg: Reg, bus: &mut Bus) {
        bus.write_mem(self.op(), self.regs.get(reg));
    }

    /// Executes a store register indirect instruction.
    pub fn store_reg_indirect(&mut self, bus: &mut Bus) {
        match source_and_target_from(self.op_lo) {
            Some((source, target)) if !source.is16() && target.is16() => {
                bus.write_mem(self.regs.get16(target), self.regs.get(source));
            }
            _ => self.trap(TRAP_ILLEGAL, bus),
        }
    }

    /// Subtract with carry.
    pub fn sub(&mut self, reg: Reg, subtrahend: u8) {
        let minuend = self.regs.get(reg);
        let (result1, borrow1) = minuend.overflowing_sub(subtrahend);
        let borrow_in = u8::from(!self.flags.carry);
        let (result2, borrow2) = result1.overflowing_sub(borrow_in);
        self.flags.carry = !(borrow1 || borrow2);
        self.regs.set(reg, result2);
        self.flags.zero = result2 == 0;
    }

    /// Executes a trap.
    ///
    /// The `trap_code` is used to select a vector from the trap table, and the CPU jumps
    /// to that address after pushing the return address and the trap code to the stack.
    pub fn trap(&mut self, mut trap_code: u8, bus: &mut Bus) {
        if trap_code == 0x20 {
            print!("{}", self.regs.get(Reg::A) as char);
        }
        if trap_code >= 0x40 {
            trap_code = TRAP_ILLEGAL;
        }
        let ret_addr = self.pc;
        let [hi, lo] = ret_addr.to_be_bytes();
        self.stack_push(lo, bus);
        self.state = WaitTrapLo(hi, trap_code);
    }
}

/// The state of the CPU's flag bits.
#[derive(Debug, Default)]
pub struct Flags {
    /// Indicates carry (from addition) or 'no borrow' (from subtraction or comparison).
    pub carry: bool,
    /// Indicates a zero result from the last operation.
    pub zero: bool,
}

/// The state of the CPU on the next tick.
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
    /// Reads a byte from memory for a `dec` instruction.
    ReadDec(u16),
    /// Reads a byte from memory for an `inc` instruction.
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
    /// Reads the high byte of the return address for a `ret` instruction.
    ReadRetHi,
    /// Reads the low byte of the return address for a `ret` instruction.
    ReadRetLo,
    /// Reads the first of two stack values from the bus.
    ReadStackHi(Reg),
    /// Reads the low byte of the selected trap vector.
    ReadTrapVecLo(u16),
    /// Waits for the high byte of the address to jump to.
    WaitAddrHi,
    /// Waits for the low byte of the return address to be pushed for a `call`
    /// instruction.
    WaitCall(u8, u16),
    /// Waits for a byte from memory for a `dec` instruction.
    WaitDec(u16),
    /// Waits for a byte from memory for an `inc` instruction.
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
    /// Waits for the high byte of the return address for a `ret` instruction.
    WaitRetHi,
    /// Waits for the low byte of the return address for a `ret` instruction.
    WaitRetLo,
    /// Waits for the first of 2 stack pops to a register.
    WaitStackHi(Reg),
    /// Waits for the trap code to be pushed following a trap.
    WaitTrapCode(u8),
    /// Waits for the high byte of the return address to be pushed following a trap.
    WaitTrapHi(u8),
    /// Waits for the low byte of the return address to be pushed following a trap.
    WaitTrapLo(u8, u8),
    /// Waits for the low byte of the trap vector following a trap.
    WaitTrapVecLo(u16),
}

impl Display for State {
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
                WaitPush(_) => "WPSH",
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
    use crate::{
        asm::{as_hex, assemble_with_debug},
        instructions::InstructionKind::Halt,
        regs::Reg::*,
        system::System,
    };

    use super::*;

    #[test]
    fn cpu_states_are_correct_for_1_byte_instruction() {
        let mut sys = System::default();
        let source = "
        nop
        halt";
        sys.mem
            .load(0x0100, &assemble_with_debug(source).unwrap())
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
        let source = "
        ld a, 0xFF
        halt";
        sys.mem
            .load(0x0100, &assemble_with_debug(source).unwrap())
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
        assert_eq!(sys.cpu.regs.get(A), 0xFF);
        assert_eq!(sys.cpu.pc, 0x0102);
    }

    #[test]
    fn cpu_states_are_correct_for_3_byte_instruction() {
        let mut sys = System::default();
        let source = "
        ld ab, 0xBEEF
        halt";
        sys.mem
            .load(0x0100, &assemble_with_debug(source).unwrap())
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
        assert_eq!(sys.cpu.regs.get16(AB), 0xBEEF);
        assert_eq!(sys.cpu.pc, 0x0103);
    }

    #[test]
    fn cpu_states_are_correct_for_mem_read_instruction() {
        let mut sys = System::default();
        let source = "
        ld b, (cd)
        halt";
        sys.mem.set(0x0110, 0xFF);
        sys.cpu.regs.set16(Reg::CD, 0x0110);
        sys.mem
            .load(0x0100, &assemble_with_debug(source).unwrap())
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
        assert_eq!(sys.cpu.regs.get(B), 0xFF);
        assert_eq!(sys.cpu.pc, 0x0102);
        assert_eq!(sys.cpu.state, FetchOpcode);
    }

    #[test]
    fn cpu_states_are_correct_for_mem_write_instruction() {
        let mut sys = System::default();
        let source = "
        ld 0xBEEF, a
        halt";
        sys.mem
            .load(0x0100, &assemble_with_debug(source).unwrap())
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
        let source = "
        pop cd
        halt";
        sys.mem.load(0xBFFE, &[0xBA, 0xBE]).unwrap();
        sys.cpu.regs.set16(SP, 0xBFFD);
        sys.mem
            .load(0x0100, &assemble_with_debug(source).unwrap())
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
        let source = "
        push ab
        halt";
        sys.cpu.regs.set16(SP, 0xBFFF);
        sys.cpu.regs.set16(AB, 0xCAFE);
        sys.mem
            .load(0x0100, &assemble_with_debug(source).unwrap())
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

    #[test]
    fn cpu_traps_for_various_illegal_programs() {
        let mut sys = System::default();
        // dummy trap 0x00 handler, jumps to `halt` at 0x0002
        sys.mem.load(0x0000, &[0x02, 0x00, u8::from(Halt)]).unwrap();
        let cases: &[&[u8]] = &[
            &[0x01, 0xFF], // reserved opcode
            &[0x1D, 0xFF], // `ld R, (RR)` with invalid regs
            &[0x28, 0xFF], // `ld (RR), R` with invalid regs
            &[0x1F, 0x08], // `ld R, R` with mixed 8/16 regs
            &[0x3D, 0xFF], // `inc (RR)` with invalid regs
            &[0x4D, 0xFF], // `dec (RR)` with invalid regs
            &[0xF9, 0x40], // `trap` with invalid code
        ];
        for prog in cases {
            // initialise stack
            sys.cpu.regs.set16(SP, 0xBFFF);
            // junk to be overwritten by trap stack frame
            sys.mem.load(0xBFFD, &[0xFF, 0xFF, 0xFF]).unwrap();
            sys.test_prog(prog);
            // verify trap stack frame
            assert_eq!(
                sys.mem.get(0xBFFD),
                TRAP_ILLEGAL,
                "{}: wrong trap code",
                as_hex(prog)
            );
            assert_eq!(
                sys.mem.get(0xBFFE),
                0x01,
                "{}: wrong return address high byte",
                as_hex(prog)
            );
            assert_eq!(
                sys.mem.get(0xBFFF),
                u8::try_from(prog.len()).unwrap(),
                "{}: wrong return address low byte",
                as_hex(prog)
            );
        }
    }

    #[expect(clippy::bool_assert_comparison, reason = "clarity")]
    #[test]
    fn reset_resets_cpu() {
        let mut sys = System::default();
        sys.cpu.regs.set16(AB, 0xBEEF);
        sys.cpu.regs.set16(SP, 0xFFFD);
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
        assert_eq!(sys.cpu.regs.get16(AB), 0x0000, "AB not reset");
        assert_eq!(sys.cpu.regs.get16(SP), 0x0000, "SP not reset");
        assert_eq!(sys.cpu.pc, 0xC000, "PC not initialized from reset vector");
        assert_eq!(sys.cpu.flags.carry, false, "carry not reset");
        assert_eq!(sys.cpu.flags.zero, false, "zero not reset");
    }
}
