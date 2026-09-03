use core::fmt::{Display, Formatter};

use anyhow::bail;

use crate::regs::Reg;

/// Instruction kinds.
#[derive(Copy, Clone, Debug)]
pub enum InstructionKind {
    /// Add with carry.
    Add(Reg),
    /// Bitwise immediate AND.
    And(Reg),
    /// Branch always.
    BranchAlways,
    /// Branch if the carry flag is clear.
    BranchCc,
    /// Branch if the carry flag is set.
    BranchCs,
    /// Branch if the zero flag is set.
    BranchEq,
    /// Branch if the zero flag is clear.
    BranchNe,
    /// Call a subroutine.
    Call,
    /// Clear carry flag.
    Clc,
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
    /// Far jump.
    Jmp,
    /// Load a register with an immediate operand.
    LdRegImm(Reg),
    /// Load a register from an indirect address in another register.
    LdRegIndirect,
    /// Load a register from another register.
    LdRegReg,
    /// Logical shift right.
    Lsr(Reg),
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
    /// Set carry flag.
    Sec,
    /// Store a register value at an immediate address.
    StoreRegDirect(Reg),
    /// Store a register value at an indirect address in another register.
    StoreRegIndirect,
    /// Subtract with carry.
    Sub(Reg),
    /// Trap with a specified code.
    Trap,
}

use InstructionKind::*;

impl Display for InstructionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Add(reg) => format!("add {reg}, N"),
                And(reg) => format!("and {reg}, N"),
                BranchAlways => "bra D".to_owned(),
                BranchCc => "bcc D".to_owned(),
                BranchCs => "bcs D".to_owned(),
                BranchEq => "beq D".to_owned(),
                BranchNe => "bne D".to_owned(),
                Call => "call NN".to_owned(),
                Clc => "clc".to_owned(),
                Cmp(reg) => format!("cmp {reg}, N"),
                Dec(reg) => format!("dec {reg}"),
                DecIndirect => "dec (RR)".to_owned(),
                DecMem => "dec (NN)".to_owned(),
                Halt => "halt".to_owned(),
                Inc(reg) => format!("inc {reg}"),
                IncIndirect => "inc (RR)".to_owned(),
                IncMem => "inc (NN)".to_owned(),
                Jmp => "jmp NN".to_owned(),
                LdRegImm(reg) => format!("ld {reg}, {}", if reg.is16() { "NN" } else { "N" }),
                LdRegIndirect => "ld R, (RR)".to_owned(),
                LdRegReg => "ld R, R".to_owned(),
                Lsr(reg) => format!("lsr {reg}, S"),
                Nop => "nop".to_owned(),
                Pop(reg) => format!("pop {reg}"),
                Push(reg) => format!("push {reg}"),
                Ret => "ret".to_owned(),
                Rti => "rti".to_owned(),
                Sec => "sec".to_owned(),
                StoreRegDirect(reg) => format!("ld NN, {reg}"),
                StoreRegIndirect => "ld (RR), R".to_owned(),
                Sub(reg) => format!("sub {reg}, N"),
                Trap => "trap T".to_owned(),
            }
        )
    }
}

impl TryFrom<u8> for InstructionKind {
    type Error = anyhow::Error;

    fn try_from(opcode: u8) -> Result<Self, Self::Error> {
        // Register ID for instructions with X0.. opcodes.
        let reg = Reg::try_from(opcode & 0x0F);
        // Register ID for instructions with X8..XF opcodes.
        let reg2 = Reg::try_from(opcode.wrapping_sub(8) & 0x0F);
        Ok(match opcode {
            0x00 => Halt,
            0x01 => Nop,
            0x03 => Sec,
            0x04 => Clc,
            0x08 => Ret,
            0x09 => Rti,
            0x10..=0x1C => LdRegImm(reg?),
            0x1D => LdRegIndirect,
            0x1F => LdRegReg,
            0x20..=0x27 => StoreRegDirect(reg?),
            0x28 => StoreRegIndirect,
            0x30..=0x3C => Inc(reg?),
            0x3D => IncIndirect,
            0x3E => IncMem,
            0x40..=0x4C => Dec(reg?),
            0x4D => DecIndirect,
            0x4E => DecMem,
            0x50..=0x57 => Add(reg?),
            0x60..=0x67 => Sub(reg?),
            0x70..=0x7B => Cmp(reg?),
            0x80..=0x87 => And(reg?),
            0xA8..=0xAF => Lsr(reg2?),
            0xD0..=0xDB => Push(reg?),
            0xE0..=0xEB => Pop(reg?),
            0xF0 => BranchAlways,
            0xF1 => BranchEq,
            0xF2 => BranchNe,
            0xF3 => BranchCs,
            0xF4 => BranchCc,
            0xF7 => Jmp,
            0xF8 => Call,
            0xF9 => Trap,
            _ => bail!("invalid opcode {opcode}"),
        })
    }
}

impl From<InstructionKind> for u8 {
    fn from(ins: InstructionKind) -> Self {
        match ins {
            Add(reg) => 0x50 | u8::from(reg),
            And(reg) => 0x80 | u8::from(reg),
            BranchAlways => 0xF0,
            BranchCc => 0xF4,
            BranchCs => 0xF3,
            BranchEq => 0xF1,
            BranchNe => 0xF2,
            Call => 0xF8,
            Clc => 0x04,
            Cmp(reg) => 0x70 | u8::from(reg),
            Dec(reg) => 0x40 | u8::from(reg),
            DecIndirect => 0x4D,
            DecMem => 0x4E,
            Halt => 0x00,
            Jmp => 0xF7,
            Inc(reg) => 0x30 | u8::from(reg),
            IncIndirect => 0x3D,
            IncMem => 0x3E,
            LdRegImm(reg) => 0x10 | u8::from(reg),
            LdRegIndirect => 0x1D,
            LdRegReg => 0x1F,
            Lsr(reg) => 0xA8 | u8::from(reg),
            Nop => 0x01,
            Pop(reg) => 0xE0 | u8::from(reg),
            Push(reg) => 0xD0 | u8::from(reg),
            Ret => 0x08,
            Rti => 0x09,
            Sec => 0x03,
            StoreRegDirect(reg) => 0x20 | u8::from(reg),
            StoreRegIndirect => 0x28,
            Sub(reg) => 0x60 | u8::from(reg),
            Trap => 0xF9,
        }
    }
}

impl InstructionKind {
    /// Returns the number of operands this instruction takes.
    #[must_use]
    pub fn operands(&self) -> Operands {
        use Operands::*;
        match *self {
            Clc | Dec(_) | Halt | Inc(_) | Nop | Push(_) | Pop(_) | Ret | Rti | Sec => Zero,
            Add(_) | And(_) | BranchAlways | BranchCc | BranchCs | BranchEq | BranchNe
            | DecIndirect | IncIndirect | LdRegIndirect | LdRegReg | Lsr(_) | StoreRegIndirect
            | Sub(_) | Trap => One,
            Cmp(reg) | LdRegImm(reg) => {
                if reg.is16() {
                    Two
                } else {
                    One
                }
            }
            Call | DecMem | IncMem | Jmp | StoreRegDirect(_) => Two,
        }
    }
}

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
