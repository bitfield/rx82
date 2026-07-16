use anyhow::anyhow;

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
    /// Compare a register with an immediate operand.
    Cmp(Reg),
    /// Decrement a register.
    Dec(Reg),
    /// Halt the CPU.
    Halt,
    /// Increment a register.
    Inc(Reg),
    /// Load a register with an immediate operand.
    LdRegImm(Reg),
    /// Load a register from an indirect address in another register.
    LdRegIndirect,
    /// No operation.
    Nop,
    /// Store a register value at an immediate address.
    StoreRegDirect(Reg),
    /// Store a register value at an indirect address in another register.
    StoreRegIndirect,
}

impl TryFrom<u8> for InstructionKind {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(opcode: u8) -> Result<Self, Self::Error> {
        use InstructionKind::*;
        let reg = Reg::try_from(opcode & 0x0F);
        match opcode {
            0x00 => Ok(Halt),
            0x01 => Ok(Nop),
            0x10..=0x1B => Ok(LdRegImm(reg?)),
            0x20..=0x27 => Ok(StoreRegDirect(reg?)),
            0x2D => Ok(LdRegIndirect),
            0x30..=0x3B => Ok(Inc(reg?)),
            0x3D => Ok(StoreRegIndirect),
            0x40..=0x4B => Ok(Dec(reg?)),
            0x70..=0x7B => Ok(Cmp(reg?)),
            0xB0 => Ok(BranchAlways),
            0xB1 => Ok(BranchEq),
            0xB2 => Ok(BranchNe),
            _ => Err(anyhow!("invalid opcode {opcode}")),
        }
    }
}

impl From<InstructionKind> for u8 {
    #[inline]
    fn from(ins: InstructionKind) -> Self {
        use InstructionKind::*;
        match ins {
            BranchAlways => 0xB0,
            BranchEq => 0xB1,
            BranchNe => 0xB2,
            Cmp(reg) => 0x70 | u8::from(reg),
            Dec(reg) => 0x40 | u8::from(reg),
            Halt => 0x00,
            Inc(reg) => 0x30 | u8::from(reg),
            LdRegImm(reg) => 0x10 | u8::from(reg),
            LdRegIndirect => 0x2D,
            Nop => 0x01,
            StoreRegDirect(reg) => 0x20 | u8::from(reg),
            StoreRegIndirect => 0x3D,
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
            Cmp(reg) => cpu.cmp(reg, cpu.op()),
            Dec(reg) => cpu.decrement(reg),
            Halt => cpu.halt(),
            Inc(reg) => cpu.increment(reg),
            LdRegImm(reg) => _ = cpu.regs.set(reg, cpu.op()),
            LdRegIndirect => cpu.ld_reg_indirect(bus),
            StoreRegDirect(reg) => cpu.store_reg_direct(reg, bus),
            StoreRegIndirect => cpu.store_reg_indirect(bus),
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
            Dec(_) | Halt | Inc(_) | Nop => Zero,
            BranchAlways | BranchEq | BranchNe | LdRegIndirect | StoreRegIndirect => One,
            Cmp(reg) | LdRegImm(reg) => {
                if reg.is16() {
                    Two
                } else {
                    One
                }
            }
            StoreRegDirect(_) => Two,
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
mod tests {
    use crate::{asm::asm, regs::Reg::*, system::System};

    #[test]
    fn beq() {
        let mut sys = System::default();
        sys.cpu.flags.zero = true;
        sys.run_program(&asm("
            beq 0x00
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0003, "wrong PC after zero branch");
        sys.run_program(&asm("
            beq 0x7F
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0082, "wrong PC after max forward branch");
        sys.run_program(&asm("
            beq 0x80
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0xFF83, "wrong PC after max backward branch");
        sys.run_program(&asm("
            beq 0x01
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0004, "forward branch not taken");
        sys.run_program(&asm("
            beq 0x01
            halt
            beq 0xFD"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0003, "backward branch not taken");
        sys.run_program(&asm("
            beq 0x01
            halt
            inc a
            beq 0xFC"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0007, "backward branch taken");
        sys.run_program(&asm("
            inc a
            beq 0x01
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0004, "forward branch taken");
    }

    #[test]
    fn bne() {
        let mut sys = System::default();
        sys.cpu.flags.zero = true;
        sys.run_program(&asm("
            bne 0x01
            halt
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0003, "branch taken");
        sys.run_program(&asm("
            inc a
            bne 0x01
            halt
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0005, "branch not taken");
    }

    #[test]
    fn bra() {
        let mut sys = System::default();
        sys.cpu.flags.zero = true;
        sys.run_program(&asm("
            bra 0x01
            halt
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0004, "branch not taken");
    }

    #[test]
    fn cmp() {
        let mut sys = System::default();
        sys.cpu.flags.zero = false;
        sys.cpu.flags.carry = false;
        sys.run_program(&asm("
            ld a, 0x01
            cmp a, 0x01
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: equal cmp");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: equal cmp");
        sys.run_program(&asm("
            ld a, 0x03
            cmp a, 0x07
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal cmp");
        assert_eq!(sys.cpu.flags.carry, false, "carry set: cmp with borrow");
        sys.run_program(&asm("
            ld a, 0x07
            cmp a, 0x03
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal comparison");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: cmp with no borrow");
        sys.run_program(&asm("
            ld gh, 0xFF03
            cmp gh, 0xFF03
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: equal cmp");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: equal cmp");
        sys.run_program(&asm("
            ld ab, 0x0003
            cmp ab, 0xFF07
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal cmp");
        assert_eq!(sys.cpu.flags.carry, false, "carry set: cmp with borrow");
        sys.run_program(&asm("
            ld cd, 0x0107
            cmp cd, 0x0103
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.flags.zero, false, "zero set: unequal cmp");
        assert_eq!(sys.cpu.flags.carry, true, "carry clear: cmp with no borrow");
        sys.run_program(&asm("
            ld cd, 0xFFFF
            cmp a, 0x00
            halt"))
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
        sys.run_program(&asm("
            dec a
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0x00FF, "wrong A");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: dec to non-zero");
        sys.run_program(&asm("
            ld ef, 0xFF01
            dec ef
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(EF), 0xFF00, "wrong EF");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: dec to non-zero");
        sys.run_program(&asm("
            ld a, 0x01
            dec a
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0x0000, "wrong A");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: dec to zero");
        sys.run_program(&asm("
            ld ef, 0x0001
            dec ef
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: dec to zero");
    }

    #[test]
    fn halt() {
        let mut sys = System::default();
        sys.run_program(&asm("halt")).unwrap();
        assert!(sys.cpu.halt, "not halted");
        assert_eq!(sys.cpu.pc, 0x0001, "wrong PC");
    }

    #[test]
    fn inc() {
        let mut sys = System::default();
        sys.run_program(&asm("
            inc d
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(D), 0x0001, "wrong D");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: inc to non-zero");
        sys.run_program(&asm("
            inc ab
            halt"))
            .unwrap();
        sys.debug_print();
        assert_eq!(sys.cpu.regs.get(AB), 0x0001, "wrong AB");
        assert_eq!(sys.cpu.flags.zero, false, "zero set: inc to non-zero");
        sys.run_program(&asm("
            ld a, 0xFF
            inc a
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0x0000, "wrong A");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: inc zero");
        sys.run_program(&asm("
            ld ab, 0xFFFF
            inc ab
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(AB), 0x0000, "wrong AB");
        assert_eq!(sys.cpu.flags.zero, true, "zero clear: inc zero");
    }

    #[test]
    fn ld_reg_imm8() {
        let mut sys = System::default();
        sys.run_program(&asm("
            ld a, 0xFF
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(A), 0x00FF, "wrong A");
        assert_eq!(sys.cpu.pc, 0x0003, "wrong PC");
    }

    #[test]
    fn ld_reg_imm16() {
        let mut sys = System::default();
        sys.run_program(&asm("
            ld ab, 0x00C0
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.regs.get(AB), 0x00C0, "wrong AB");
        assert_eq!(sys.cpu.pc, 0x0004, "wrong PC");
    }

    #[test]
    fn ld_reg_indirect() {
        let mut sys = System::default();
        sys.run_program(&asm("
            ld a, 0xFF
            ld 0x0100, a
            ld cd, 0x0100
            ld b, (cd)
            halt"))
            .unwrap();
        sys.debug_print();
        assert_eq!(sys.cpu.regs.get(B), 0xFF, "wrong B");
    }

    #[test]
    fn nop() {
        let mut sys = System::default();
        sys.run_program(&asm("
            nop
            halt"))
            .unwrap();
        assert_eq!(sys.cpu.pc, 0x0002, "wrong PC");
    }

    #[test]
    fn store_reg_direct() {
        let mut sys = System::default();
        sys.run_program(&asm("
            ld a, 0xFF
            ld 0xBEEF, a
            halt"))
            .unwrap();
        let val = sys.mem.get(0xBEEF);
        assert_eq!(val, 0xFF, "wrong mem value");
    }

    #[test]
    fn store_reg_indirect() {
        let mut sys = System::default();
        sys.run_program(&asm("
            ld ef, 0xBABE
            ld a, 0xFF
            ld (ef), a
            halt"))
            .unwrap();
        let val = sys.mem.get(0xBABE);
        assert_eq!(val, 0xFF, "wrong mem value");
    }

    #[test]
    fn zero_flag() {
        let mut sys = System::default();
        assert_eq!(sys.cpu.flags.zero, false, "zero flag wrongly initialised");
        sys.run_program(&asm("
            dec a
            halt"))
            .unwrap(); // a = -1
        assert_eq!(sys.cpu.flags.zero, false, "zero flag set after dec");
        sys.run_program(&asm("
            inc a
            halt"))
            .unwrap(); // a = 0
        assert_eq!(sys.cpu.flags.zero, true, "zero flag clear after inc");
        sys.run_program(&asm("
            inc a
            halt"))
            .unwrap(); // a = 1
        assert_eq!(sys.cpu.flags.zero, false, "zero flag set after inc");
        sys.run_program(&asm("
            dec a
            halt"))
            .unwrap(); // a = 0
        assert_eq!(sys.cpu.flags.zero, true, "zero flag clear after dec");
    }
}
