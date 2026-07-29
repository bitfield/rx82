use anyhow::bail;

use crate::{bus::Bus, cpu::Cpu, regs::Reg};

/// Instruction kinds.
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum InstructionKind {
    /// Branch always.
    BranchAlways,
    /// Branch if the zero flag is set.
    BranchEq,
    /// Branch if the zero flag is clear.
    BranchNe,
    /// Call a subroutine.
    Call,
    /// Compare a register with an immediate operand.
    Cmp(Reg),
    /// Decrement a register.
    Dec(Reg),
    /// Decrement a memory location in a register.
    DecIndirect,
    /// Decrement a memory location.
    DecMem,
    /// Halt the CPU.
    Halt,
    /// Increment a register.
    Inc(Reg),
    /// Increment a memory location in a register.
    IncIndirect,
    /// Increment a memory location.
    IncMem,
    /// Load a register with an immediate operand.
    LdRegImm(Reg),
    /// Load a register from an indirect address in another register.
    LdRegIndirect,
    /// Load a register from another register.
    LdRegReg,
    /// No operation.
    Nop,
    /// Pop a register value from the stack.
    Pop(Reg),
    /// Push a register value to the stack.
    Push(Reg),
    /// Return from a subroutine call.
    Ret,
    /// Return from a trap.
    Rti,
    /// Store a register value at an immediate address.
    StoreRegDirect(Reg),
    /// Store a register value at an indirect address in another register.
    StoreRegIndirect,
    /// Trap with a specified code.
    Trap,
}

impl TryFrom<u8> for InstructionKind {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(opcode: u8) -> Result<Self, Self::Error> {
        use InstructionKind::*;
        let reg = Reg::try_from(opcode & 0x0F);
        Ok(match opcode {
            0x00 => Halt,
            0x01 => Nop,
            0x08 => Ret,
            0x09 => Rti,
            0x10..=0x1C => LdRegImm(reg?),
            0x20..=0x27 => StoreRegDirect(reg?),
            0x2D => LdRegIndirect,
            0x2E => StoreRegIndirect,
            0x2F => LdRegReg,
            0x30..=0x3C => Inc(reg?),
            0x3D => IncIndirect,
            0x3E => IncMem,
            0x40..=0x4C => Dec(reg?),
            0x4D => DecIndirect,
            0x4E => DecMem,
            0x70..=0x7B => Cmp(reg?),
            0xD0..=0xDB => Push(reg?),
            0xE0..=0xEB => Pop(reg?),
            0xF0 => BranchAlways,
            0xF1 => BranchEq,
            0xF2 => BranchNe,
            0xF8 => Call,
            0xF9 => Trap,
            _ => bail!("invalid opcode {opcode}"),
        })
    }
}

impl From<InstructionKind> for u8 {
    #[inline]
    fn from(ins: InstructionKind) -> Self {
        use InstructionKind::*;
        match ins {
            BranchAlways => 0xF0,
            BranchEq => 0xF1,
            BranchNe => 0xF2,
            Call => 0xF8,
            Cmp(reg) => 0x70 | u8::from(reg),
            Dec(reg) => 0x40 | u8::from(reg),
            DecIndirect => 0x4D,
            DecMem => 0x4E,
            Halt => 0x00,
            Inc(reg) => 0x30 | u8::from(reg),
            IncIndirect => 0x3D,
            IncMem => 0x3E,
            LdRegImm(reg) => 0x10 | u8::from(reg),
            LdRegIndirect => 0x2D,
            LdRegReg => 0x2F,
            Nop => 0x01,
            Pop(reg) => 0xE0 | u8::from(reg),
            Push(reg) => 0xD0 | u8::from(reg),
            Ret => 0x08,
            Rti => 0x09,
            StoreRegDirect(reg) => 0x20 | u8::from(reg),
            StoreRegIndirect => 0x2E,
            Trap => 0xF9,
        }
    }
}

impl InstructionKind {
    /// Executes the instruction.
    #[inline]
    pub fn execute(&self, cpu: &mut Cpu, bus: &mut Bus) {
        use InstructionKind::*;
        match *self {
            BranchAlways => cpu.branch(cpu.op_lo),
            BranchEq if cpu.flags.zero => cpu.branch(cpu.op_lo),
            BranchNe if !cpu.flags.zero => cpu.branch(cpu.op_lo),
            Call => cpu.call(cpu.op(), bus),
            Cmp(reg) => cpu.cmp(reg, cpu.op()),
            Dec(reg) => cpu.decrement(reg),
            DecIndirect => cpu.dec_indirect(bus),
            DecMem => cpu.dec_mem(cpu.op(), bus),
            Halt => cpu.halt(),
            Inc(reg) => cpu.increment(reg),
            IncIndirect => cpu.inc_indirect(bus),
            IncMem => cpu.inc_mem(cpu.op(), bus),
            LdRegImm(reg) => _ = cpu.regs.set(reg, cpu.op()),
            LdRegIndirect => cpu.ld_reg_indirect(bus),
            LdRegReg => cpu.ld_reg_reg(bus),
            Pop(reg) => cpu.pop(reg, bus),
            Push(reg) => cpu.push(reg, bus),
            Ret => cpu.ret(bus),
            Rti => cpu.rti(bus),
            StoreRegDirect(reg) => cpu.store_reg_direct(reg, bus),
            StoreRegIndirect => cpu.store_reg_indirect(bus),
            Trap => cpu.trap(cpu.op_lo, bus),
            Nop | BranchEq | BranchNe => {}
        }
    }

    /// Returns the number of operands this instruction takes.
    #[inline]
    #[must_use]
    pub fn operands(&self) -> Operands {
        use InstructionKind::*;
        use Operands::*;
        match *self {
            Dec(_) | Halt | Inc(_) | Nop | Push(_) | Pop(_) | Ret | Rti => Zero,
            BranchAlways | BranchEq | BranchNe | DecIndirect | IncIndirect | LdRegIndirect
            | LdRegReg | StoreRegIndirect | Trap => One,
            Cmp(reg) | LdRegImm(reg) => {
                if reg.is16() {
                    Two
                } else {
                    One
                }
            }
            Call | DecMem | IncMem | StoreRegDirect(_) => Two,
        }
    }
}

#[expect(clippy::exhaustive_enums, reason = "this actually is exhaustive")]
/// Specifies whether an instruction takes zero, one, or two operands.
#[derive(Clone, Debug, PartialEq)]
pub enum Operands {
    /// One operand.
    One,
    /// Two operands.
    Two,
    /// Zero operands.
    Zero,
}

#[cfg(test)]
#[expect(clippy::bool_assert_comparison, reason = "clarity")]
#[expect(clippy::unwrap_used, reason = "test")]
#[expect(clippy::default_numeric_fallback, reason = "hex literals")]
mod tests {
    use crate::{asm::assemble, regs::Reg::*, system::System};

    macro_rules! assert_hex {
        ( $got:expr, $want:expr, $msg:expr ) => {
            assert_eq!(
                $got, $want,
                "{}: want {:#06X}, got {:#06X}",
                $msg, $want, $got,
            );
        };
    }

    #[test]
    fn beq() {
        let mut sys = System::default();
        sys.cpu.flags.zero = true;
        sys.trace_program(&assemble(
            "
            beq 0x00
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0103, "wrong PC after zero branch");
        sys.trace_program(&assemble(
            "
            beq 0x7F
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0182, "wrong PC after max forward branch");
        sys.mem
            .load(
                0x1000,
                &assemble(
                    "
                beq 0x80
                halt",
                ),
            )
            .unwrap();
        sys.cpu.pc = 0x1000;
        sys.run();
        assert_hex!(sys.cpu.pc, 0x0F83, "wrong PC after max backward branch");
        sys.trace_program(&assemble(
            "
            beq 0x01
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0104, "forward branch not taken");
        sys.trace_program(&assemble(
            "
            beq 0x01
            halt
            beq 0xFD",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0103, "backward branch not taken");
        sys.trace_program(&assemble(
            "
            beq 0x01
            halt
            inc a
            beq 0xFC",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0107, "backward branch taken");
        sys.trace_program(&assemble(
            "
            inc a
            beq 0x01
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0104, "forward branch taken");
    }

    #[test]
    fn bne() {
        let mut sys = System::default();
        sys.cpu.flags.zero = true;
        sys.trace_program(&assemble(
            "
            bne 0x01
            halt
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0103, "branch taken");
        sys.trace_program(&assemble(
            "
            inc a
            bne 0x01
            halt
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0105, "branch not taken");
    }

    #[test]
    fn bra() {
        let mut sys = System::default();
        sys.cpu.flags.zero = true;
        sys.trace_program(&assemble(
            "
            bra 0x01
            halt
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0104, "branch not taken");
    }

    #[test]
    fn call() {
        let mut sys = System::default();
        sys.cpu.regs.set(SP, 0x0200);
        sys.trace_program(&assemble(
            "
            call SUBR
            halt
        SUBR:
            ld a, 0xFF
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0107, "wrong PC");
        assert_hex!(sys.cpu.regs.get(A), 0xFF, "wrong A");
        assert_hex!(sys.peek_mem(0x01FF), 0x01, "wrong high byte on stack");
        assert_hex!(sys.peek_mem(0x0200), 0x03, "wrong low byte on stack");
        assert_hex!(sys.cpu.regs.get(SP), 0x01FE, "wrong SP");
    }

    #[test]
    fn cmp() {
        let mut sys = System::default();
        sys.cpu.flags.zero = false;
        sys.cpu.flags.carry = false;
        sys.trace_program(&assemble(
            "
            ld a, 0x01
            cmp a, 0x01
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: equal cmp");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: equal cmp");
        sys.trace_program(&assemble(
            "
            ld a, 0x03
            cmp a, 0x07
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal cmp");
        assert_eq!(sys.cpu.flags.carry, false, "carry set: cmp with borrow");
        sys.trace_program(&assemble(
            "
            ld a, 0x07
            cmp a, 0x03
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal comparison");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: cmp with no borrow");
        sys.trace_program(&assemble(
            "
            ld gh, 0xFF03
            cmp gh, 0xFF03
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: equal cmp");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: equal cmp");
        sys.trace_program(&assemble(
            "
            ld ab, 0x0003
            cmp ab, 0xFF07
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal cmp");
        assert_eq!(sys.cpu.flags.carry, false, "carry set: cmp with borrow");
        sys.trace_program(&assemble(
            "
            ld cd, 0x0107
            cmp cd, 0x0103
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal cmp");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: cmp with no borrow");
        sys.trace_program(&assemble(
            "
            ld cd, 0xFFFF
            cmp a, 0x00
            halt",
        ))
        .unwrap();
        assert_eq!(
            sys.cpu.flags.zero, true,
            "zero clear: equal cmp (junk in high byte?)"
        );
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: cmp with no borrow");
    }

    #[test]
    fn dec() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            dec a
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(A), 0x00FF, "wrong A");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: dec to non-zero");
        sys.trace_program(&assemble(
            "
            ld sp, 0xFF01
            dec sp
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(SP), 0xFF00, "wrong SP");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: dec to non-zero");
        sys.trace_program(&assemble(
            "
            ld a, 0x01
            dec a
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(A), 0x0000, "wrong A");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: dec to zero");
        sys.trace_program(&assemble(
            "
            ld ef, 0x0001
            dec ef
            halt",
        ))
        .unwrap();
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: dec to zero");
    }

    #[test]
    fn dec_nn() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0x02
            ld 0x0010, a
            dec (0x0010)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0x01, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: dec to non-zero");
        sys.trace_program(&assemble(
            "
            dec (0x0010)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0x00, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: dec to zero");
    }

    #[test]
    fn dec_rr() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0x02
            ld ef, 0x0010
            ld (ef), a
            dec (ef)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0x01, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: dec to non-zero");
        sys.trace_program(&assemble(
            "
            dec (ef)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0x00, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: dec to zero");
    }

    #[test]
    fn halt() {
        let mut sys = System::default();
        sys.trace_program(&assemble("halt")).unwrap();
        assert!(sys.cpu.halt, "not halted");
        assert_hex!(sys.cpu.pc, 0x0101, "wrong PC");
    }

    #[test]
    fn inc() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            inc d
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(D), 0x0001, "wrong D");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: inc to non-zero");
        sys.trace_program(&assemble(
            "
            inc ab
            halt",
        ))
        .unwrap();
        sys.debug_print();
        assert_hex!(sys.cpu.regs.get(AB), 0x0001, "wrong AB");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: inc to non-zero");
        sys.trace_program(&assemble(
            "
            ld a, 0xFF
            inc a
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(A), 0x0000, "wrong A");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: inc to zero");
        sys.trace_program(&assemble(
            "
            ld ab, 0xFFFF
            inc ab
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(AB), 0x0000, "wrong AB");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: inc to zero");
        sys.trace_program(&assemble(
            "
            ld sp, 0xFFFF
            inc sp
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(SP), 0x0000, "wrong SP");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: inc to zero");
    }

    #[test]
    fn inc_nn() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0xFE
            ld 0x0010, a
            inc (0x0010)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0xFF, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: inc to non-zero");
        sys.trace_program(&assemble(
            "
            inc (0x0010)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0x00, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: inc to zero");
    }

    #[test]
    fn inc_rr() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0xFE
            ld cd, 0x0010
            ld (cd), a
            inc (cd)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0xFF, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: inc to non-zero");
        sys.trace_program(&assemble(
            "
            inc (cd)
            halt",
        ))
        .unwrap();
        assert_hex!(sys.mem.get(0x0010), 0x00, "wrong memory contents");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: inc to zero");
    }

    #[test]
    fn ld_reg_imm8() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0xFF
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(A), 0x00FF, "wrong A");
        assert_hex!(sys.cpu.pc, 0x0103, "wrong PC");
    }

    #[test]
    fn ld_reg_imm16() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld ab, 0x00C0
            ld sp, 0xBEEF
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(AB), 0x00C0, "wrong AB");
        assert_hex!(sys.cpu.regs.get(SP), 0xBEEF, "wrong SP");
        assert_hex!(sys.cpu.pc, 0x0107, "wrong PC");
    }

    #[test]
    fn ld_reg_indirect() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0xFF
            ld 0x0100, a
            ld cd, 0x0100
            ld b, (cd)
            ld sp, 0x0100
            ld c, (sp)
            halt",
        ))
        .unwrap();
        sys.debug_print();
        assert_hex!(sys.cpu.regs.get(B), 0xFF, "wrong B");
        assert_hex!(sys.cpu.regs.get(C), 0xFF, "wrong C");
    }

    #[test]
    fn ld_reg_reg() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0xFF
            ld b, a
            ld cd, ab
            ld e, c
            halt",
        ))
        .unwrap();
        sys.debug_print();
        assert_hex!(sys.cpu.regs.get(B), 0xFF, "wrong B");
        assert_hex!(sys.cpu.regs.get(CD), 0xFFFF, "wrong CD");
        assert_hex!(sys.cpu.regs.get(E), 0xFF, "wrong E");
    }

    #[test]
    fn nop() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            nop
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0102, "wrong PC");
    }

    #[test]
    fn pop() {
        let mut sys = System::default();
        sys.mem.load(0xBFFD, &[0x01, 0x02, 0x03]).unwrap();
        sys.trace_program(&assemble(
            "
            ld sp, 0xBFFC
            pop gh
            pop b
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(SP), 0xBFFF, "wrong SP");
        assert_hex!(sys.cpu.regs.get(GH), 0x0102, "wrong GH");
        assert_hex!(sys.cpu.regs.get(B), 0x03, "wrong B");
    }

    #[test]
    fn push() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld sp, 0xBFFF
            ld a, 0xFF
            push a
            ld cd, 0xCAFE
            push cd
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.regs.get(SP), 0xBFFC, "wrong SP");
        assert_hex!(sys.mem.get(0xBFFF), 0xFF, "wrong stack value for A");
        assert_hex!(sys.mem.get(0xBFFE), 0xFE, "wrong stack value for D");
        assert_hex!(sys.mem.get(0xBFFD), 0xCA, "wrong stack value for C");
    }

    #[test]
    fn ret() {
        let mut sys = System::default();
        sys.cpu.regs.set(SP, 0x0200);
        sys.trace_program(&assemble(
            "
            call SUBR
            inc a
            halt
        SUBR:
            ld a, 0x01
            ret",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0105, "wrong PC");
        assert_hex!(sys.cpu.regs.get(A), 0x02, "wrong A");
        assert_hex!(sys.cpu.regs.get(SP), 0x0200, "wrong SP");
    }

    #[test]
    fn rti() {
        let mut sys = System::default();
        sys.cpu.regs.set(SP, 0x0200);
        // trap vector 0x01 points to TRAP_1
        sys.mem.load(0x0000, &[0x00, 0x00, 0x10, 0x01]).unwrap();
        sys.trace_program(&assemble(
            "
            trap 0x01
            inc a
            halt
            org 0x0110
        TRAP_1:
            pop a
            push a
            rti",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0104, "wrong PC");
        assert_hex!(sys.cpu.regs.get(A), 0x02, "wrong A");
        assert_hex!(sys.cpu.regs.get(SP), 0x0200, "wrong SP");
    }

    #[test]
    fn store_reg_direct() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld a, 0xFF
            ld 0xBEEF, a
            halt",
        ))
        .unwrap();
        let val = sys.mem.get(0xBEEF);
        assert_hex!(val, 0xFF, "wrong mem value");
    }

    #[test]
    fn store_reg_indirect() {
        let mut sys = System::default();
        sys.trace_program(&assemble(
            "
            ld ef, 0xBABE
            ld a, 0xFF
            ld (ef), a
            halt",
        ))
        .unwrap();
        let val = sys.mem.get(0xBABE);
        assert_hex!(val, 0xFF, "wrong mem value");
    }

    #[test]
    fn trap() {
        let mut sys = System::default();
        sys.cpu.regs.set(SP, 0x0200);
        // trap vector 0x01 points to TRAP_1
        sys.mem.load(0x0000, &[0x00, 0x00, 0x10, 0x01]).unwrap();
        sys.trace_program(&assemble(
            "
            trap 0x01
            halt
            org 0x0110
        TRAP_1:
            pop a
            push a
            halt",
        ))
        .unwrap();
        assert_hex!(sys.cpu.pc, 0x0113, "wrong PC");
        assert_hex!(sys.cpu.regs.get(A), 0x01, "wrong A");
        assert_hex!(sys.peek_mem(0x01FF), 0x01, "wrong high byte on stack");
        assert_hex!(sys.peek_mem(0x0200), 0x02, "wrong low byte on stack");
        assert_hex!(sys.cpu.regs.get(SP), 0x01FD, "wrong SP");
    }

    #[test]
    fn zero_flag() {
        let mut sys = System::default();
        assert_eq!(sys.cpu.flags.zero, false, "zero flag wrongly initialised");
        sys.trace_program(&assemble(
            "
            dec a
            halt",
        ))
        .unwrap(); // a = -1
        assert_eq!(sys.cpu.flags.zero, false, "zero flag set after dec");
        sys.trace_program(&assemble(
            "
            inc a
            halt",
        ))
        .unwrap(); // a = 0
        assert_eq!(sys.cpu.flags.zero, true, "zero flag clear after inc");
        sys.trace_program(&assemble(
            "
            inc a
            halt",
        ))
        .unwrap(); // a = 1
        assert_eq!(sys.cpu.flags.zero, false, "zero flag set after inc");
        sys.trace_program(&assemble(
            "
            dec a
            halt",
        ))
        .unwrap(); // a = 0
        assert_eq!(sys.cpu.flags.zero, true, "zero flag clear after dec");
    }
}
