use anyhow::{Context as _, Result, anyhow, bail};

use core::{
    fmt::{Debug, Display, Formatter},
    iter::{self, Peekable},
    slice::Iter,
    str::{Chars, FromStr as _},
};
use std::{collections::HashMap, fs, path::Path};

use crate::{
    instructions::InstructionKind::{self, *},
    regs::{self, Reg, source_and_target_from, source_from, u8_from},
};

use Token::*;

/// The default base address for assembled code.
pub const BASE: u16 = 0x0100;

/// Keywords recognised by the assembler.
pub const KEYWORDS: &[&str] = &[
    "add", "and", "bcc", "bcs", "beq", "bne", "bra", "call", "clc", "cmp", "data", "dec", "halt",
    "inc", "jmp", "ld", "lsr", "nop", "org", "pop", "push", "ret", "rti", "sec", "sub", "trap",
];

/// Assembles a given source program.
pub struct Assembler {
    /// Generated object code.
    pub code: Vec<u8>,
    /// Next token to assemble.
    pub cursor: usize,
    /// Enables verbose debugging.
    pub debug: bool,
    /// Label table.
    pub labels: HashMap<String, u16>,
    /// Line counter.
    pub line: usize,
    /// Location counter.
    pub loc: u16,
    /// Are we on the second pass?
    pub pass2: bool,
    /// Path to source file.
    pub path: String,
    /// Tokens to assemble.
    pub tokens: Vec<Token>,
}

impl Assembler {
    /// Assembles the source code.
    ///
    /// # Errors
    ///
    /// * Syntax errors.
    pub fn assemble(&mut self) -> Result<Vec<u8>> {
        self.pass()
            .context(format!("syntax error in {}:{}", self.path, self.line))?;
        self.code.clear();
        self.pass2 = true;
        self.cursor = 0;
        self.line = 1;
        self.loc = BASE;
        self.pass()?;
        Ok(self.code.clone())
    }

    /// Assembles keyword `kw`.
    ///
    /// # Errors
    ///
    /// * Syntax errors.
    pub fn assemble_kw(&mut self, kw: &String) -> Result<()> {
        match kw.as_str() {
            "add" => self.gen_add(),
            "and" => self.gen_and(),
            "bcc" => self.gen_branch(BranchCc),
            "bcs" => self.gen_branch(BranchCs),
            "beq" => self.gen_branch(BranchEq),
            "bne" => self.gen_branch(BranchNe),
            "bra" => self.gen_branch(BranchAlways),
            "call" => self.gen_call(),
            "clc" => self.emit_byte(u8::from(Clc)),
            "cmp" => self.gen_cmp(),
            "data" => self.gen_data(),
            "dec" => self.gen_dec(),
            "halt" => self.emit_byte(u8::from(Halt)),
            "inc" => self.gen_inc(),
            "jmp" => self.gen_jmp(),
            "ld" => self.gen_ld(),
            "lsr" => self.gen_lsr(),
            "nop" => self.emit_byte(u8::from(Nop)),
            "org" => self.org(),
            "pop" => self.gen_pop(),
            "push" => self.gen_push(),
            "ret" => self.emit_byte(u8::from(Ret)),
            "rti" => self.emit_byte(u8::from(Rti)),
            "sec" => self.emit_byte(u8::from(Sec)),
            "sub" => self.gen_sub(),
            "trap" => self.gen_trap(),
            _ => unreachable!("unknown keyword '{kw}'"),
        }
    }

    /// Adds `byte` to the generated code, updating the location counter.
    ///
    /// # Errors
    ///
    /// * Code too big for (target system) memory.
    pub fn emit_byte(&mut self, byte: u8) -> Result<()> {
        self.code.push(byte);
        self.loc = self.loc.wrapping_add(1);
        if self.loc == 1 && self.code.len() > 1 {
            // We must have wrapped around past the end of memory
            bail!("code too big for memory");
        }
        Ok(())
    }

    /// Adds `word` to the generated code in little-endian order, updating the location
    /// counter.
    ///
    /// # Errors
    ///
    /// * Code too big for (target system) memory.
    pub fn emit_word(&mut self, word: u16) -> Result<()> {
        let [lo, hi] = word.to_le_bytes();
        self.emit_byte(lo)?;
        self.emit_byte(hi)
    }

    /// Consumes the specified token.
    ///
    /// # Errors
    ///
    /// * Next token does not match `expected`.
    pub fn expect(&mut self, expected: &Token) -> Result<()> {
        let token = self.next_token()?;
        if token != *expected {
            bail!("expected {expected}, got {token}")
        }
        Ok(())
    }

    /// Returns the displacement for the current branch instruction.
    ///
    /// This may be an immediate value or a label reference.
    ///
    /// # Errors
    ///
    /// * Undefined label.
    /// * No displacement.
    /// * Displacement out of range (signed byte).
    #[expect(clippy::cast_possible_truncation, reason = "code ensures valid range")]
    pub fn expect_displacement(&mut self) -> Result<u8> {
        match self.next_token()? {
            ByteLiteral(dis) => Ok(dis),
            Identifier(label) if self.pass2 => {
                let addr = self.resolve_label(&label)?;
                // Displacement is calculated relative to the address of the next
                // instruction (i.e. PC+2)
                let long_dis = addr.wrapping_sub(self.loc).wrapping_sub(2);
                // This must fit into a signed byte (-128 <= dis <= +127)
                match long_dis {
                    0x0000..=0x007F | 0xFF80..=0xFFFF => Ok(long_dis as u8),
                    _ => bail!("displacement out of range: {long_dis:#06X}"),
                }
            }
            Identifier(_) => Ok(0), // resolve for real on next pass
            other => bail!("expected label or immediate byte displacement, got {other}"),
        }
    }

    /// Returns the operand bytes for an instruction targeting `reg`.
    ///
    /// # Errors
    ///
    /// * Missing or wrong size operand.
    pub fn expect_op_for_reg(&mut self, reg: Reg) -> Result<Vec<u8>> {
        Ok(match self.next_token()? {
            ByteLiteral(operand) if !reg.is16() => vec![operand],
            WordLiteral(operand) if reg.is16() => Vec::from(operand.to_le_bytes()),
            other if reg.is16() => bail!("expected immediate word, got {other}"),
            other => bail!("expected immediate byte, got {other}"),
        })
    }

    /// Returns the register named by the next token.
    ///
    /// # Errors
    ///
    /// * Next token is not a register name.
    pub fn expect_reg(&mut self) -> Result<Reg> {
        let reg = match self.next_token()? {
            Register(reg) => reg,
            other => bail!("expected register name, got {other}"),
        };
        Ok(reg)
    }

    /// Returns the 16-bit register named by the next token.
    ///
    /// # Errors
    ///
    /// * Next token is not a 16-bit register name.
    pub fn expect_reg16(&mut self) -> Result<Reg> {
        let reg = match self.next_token()? {
            Register(reg) if reg.is16() => reg,
            Register(reg) => bail!("expected 16-bit register name, got '{reg}'"),
            other => bail!("expected register name, got {other}"),
        };
        Ok(reg)
    }

    /// Returns the 8-bit register named by the next token.
    ///
    /// # Errors
    ///
    /// * Next token is not an 8-bit register name.
    pub fn expect_reg8(&mut self) -> Result<Reg> {
        let reg = match self.next_token()? {
            Register(reg) if !reg.is16() => reg,
            Register(reg) => bail!("expected 8-bit register name, got '{reg}'"),
            other => bail!("expected register name, got {other}"),
        };
        Ok(reg)
    }

    /// Generates an add with carry instruction.
    ///
    /// # Errors
    ///
    /// * Missing register name.
    /// * Missing comma.
    /// * Missing or mis-sized operand.
    pub fn gen_add(&mut self) -> Result<()> {
        let reg = self.expect_reg8()?;
        self.expect(&Comma)?;
        self.emit_byte(u8::from(Add(reg)))?;
        let op = self.expect_op_for_reg(reg)?;
        for byte in op {
            self.emit_byte(byte)?;
        }
        Ok(())
    }

    /// Generates a bitwise immediate and instruction.
    ///
    /// # Errors
    ///
    /// * Missing register name.
    /// * Missing comma.
    /// * Missing or mis-sized operand.
    pub fn gen_and(&mut self) -> Result<()> {
        let reg = self.expect_reg8()?;
        self.expect(&Comma)?;
        self.emit_byte(u8::from(And(reg)))?;
        let op = self.expect_op_for_reg(reg)?;
        for byte in op {
            self.emit_byte(byte)?;
        }
        Ok(())
    }

    /// Generates a branch instruction of kind `kind`.
    ///
    /// # Errors
    ///
    /// * Missing or invalid displacement.
    pub fn gen_branch(&mut self, kind: InstructionKind) -> Result<()> {
        let dis = self.expect_displacement()?;
        self.emit_byte(u8::from(kind))?;
        self.emit_byte(dis)?;
        Ok(())
    }

    /// Generates a call instruction.
    ///
    /// # Errors
    ///
    /// * Missing or invalid label or address.
    pub fn gen_call(&mut self) -> Result<()> {
        let addr = match self.next_token()? {
            Identifier(label) => self.resolve_label(&label)?,
            WordLiteral(addr) => addr,
            other => bail!("expected label or address, got {other}"),
        };
        self.emit_byte(u8::from(Call))?;
        self.emit_word(addr)?;
        Ok(())
    }

    /// Generates a compare instruction.
    ///
    /// # Errors
    ///
    /// * Missing register name.
    /// * Missing comma.
    /// * Missing or mis-sized operand.
    pub fn gen_cmp(&mut self) -> Result<()> {
        let reg = self.expect_reg()?;
        self.expect(&Comma)?;
        self.emit_byte(u8::from(Cmp(reg)))?;
        let op = self.expect_op_for_reg(reg)?;
        for byte in op {
            self.emit_byte(byte)?;
        }
        Ok(())
    }

    /// Generates literal data.
    ///
    /// # Errors
    ///
    /// * Non-ASCII character in string.
    /// * Unexpected token.
    pub fn gen_data(&mut self) -> Result<()> {
        loop {
            match self.next_token()? {
                ByteLiteral(byte) => self.emit_byte(byte)?,
                Comma => {}
                StringLiteral(string) => {
                    for ch in string.chars() {
                        if ch.is_ascii() {
                            self.emit_byte(ch as u8)?;
                        } else {
                            bail!("non-ASCII character {:#04X} in string", ch as u8);
                        }
                    }
                }
                Newline => break,
                other => bail!("expected immediate byte, got {other}"),
            }
        }
        Ok(())
    }

    /// Generates a decrement instruction.
    ///
    /// # Errors
    ///
    /// * Missing or invalid register name or address.
    pub fn gen_dec(&mut self) -> Result<()> {
        match self.next_token()? {
            ParenOpen => match self.next_token()? {
                Register(source) if source.is16() => {
                    self.expect(&ParenClose)?;
                    self.emit_byte(u8::from(DecIndirect))?;
                    let reg = u8_from(source, Reg::A); // dummy target
                    self.emit_byte(reg)?;
                }
                WordLiteral(addr) => {
                    self.expect(&ParenClose)?;
                    self.emit_byte(u8::from(DecMem))?;
                    self.emit_word(addr)?;
                }
                other => bail!("expected 16-bit register or address, got {other}"),
            },
            Register(reg) => self.emit_byte(u8::from(Dec(reg)))?,
            other => bail!("expected register or indirect address, got {other}"),
        }
        Ok(())
    }

    /// Generates an increment instruction.
    ///
    /// # Errors
    ///
    /// * Missing or invalid register name or address.
    pub fn gen_inc(&mut self) -> Result<()> {
        match self.next_token()? {
            ParenOpen => match self.next_token()? {
                Register(source) if source.is16() => {
                    self.expect(&ParenClose)?;
                    self.emit_byte(u8::from(IncIndirect))?;
                    let reg = u8_from(source, Reg::A); // dummy target
                    self.emit_byte(reg)?;
                }
                WordLiteral(addr) => {
                    self.expect(&ParenClose)?;
                    self.emit_byte(u8::from(IncMem))?;
                    self.emit_word(addr)?;
                }
                other => bail!("expected 16-bit register or address, got {other}"),
            },
            Register(reg) => self.emit_byte(u8::from(Inc(reg)))?,
            other => bail!("expected register or indirect address, got {other}"),
        }
        Ok(())
    }

    /// Generates a jump instruction.
    ///
    /// # Errors
    ///
    /// * Missing or invalid label or address.
    pub fn gen_jmp(&mut self) -> Result<()> {
        let addr = match self.next_token()? {
            Identifier(label) => self.resolve_label(&label)?,
            WordLiteral(addr) => addr,
            other => bail!("expected label or address, got {other}"),
        };
        self.emit_byte(u8::from(Jmp))?;
        self.emit_word(addr)?;
        Ok(())
    }

    /// Generates a load (or store) instruction.
    ///
    /// # Errors
    ///
    /// * Syntax errors.
    pub fn gen_ld(&mut self) -> Result<()> {
        match self.next_token()? {
            ParenOpen => self.gen_store_indirect(),
            Register(target) => self.gen_ld_reg(target),
            WordLiteral(addr) => self.gen_store_direct(addr),
            other => bail!("expected register name, got {other}"),
        }
    }

    /// Generates a load register immediate or load register register instruction.
    ///
    /// # Errors
    ///
    /// * Syntax errors.
    pub fn gen_ld_reg(&mut self, target: Reg) -> Result<()> {
        self.expect(&Comma)?;
        match self.next_token()? {
            ByteLiteral(byte) => self.gen_ld_reg_imm8(target, byte),
            Identifier(label) => self.gen_ld_reg_imm_label(target, &label),
            ParenOpen => self.gen_ld_reg_indirect(target),
            Register(source) => self.gen_ld_reg_reg(source, target),
            WordLiteral(word) => self.gen_ld_reg_imm16(target, word),
            other => bail!("unexpected token {other}"),
        }
    }

    /// Generates a load register immediate word instruction.
    ///
    /// # Errors
    ///
    /// * Wrong target register width.
    pub fn gen_ld_reg_imm16(&mut self, target: Reg, word: u16) -> Result<()> {
        if !target.is16() {
            bail!("expected immediate byte, got {word:#06X}")
        }
        self.emit_byte(u8::from(LdRegImm(target)))?;
        self.emit_word(word)
    }

    /// Generates a load register immediate byte instruction.
    ///
    /// # Errors
    ///
    /// * Wrong target register width.
    pub fn gen_ld_reg_imm8(&mut self, target: Reg, byte: u8) -> Result<()> {
        if target.is16() {
            bail!("expected immediate word, got {byte:#04X}")
        }
        self.emit_byte(u8::from(LdRegImm(target)))?;
        self.emit_byte(byte)
    }

    /// Generates a load register immediate instruction with a label operand.
    ///
    /// # Errors
    ///
    /// * If the target is not a 16-bit register.
    pub fn gen_ld_reg_imm_label(&mut self, target: Reg, label: &String) -> Result<()> {
        if !target.is16() {
            bail!("expected immediate byte, got label '{label}'")
        }
        self.emit_byte(u8::from(LdRegImm(target)))?;
        self.emit_word(self.resolve_label(label)?)
    }

    /// Generates a load register indirect instruction.
    ///
    /// # Errors
    ///
    /// * Invalid source or target registers.
    /// * Syntax errors.
    pub fn gen_ld_reg_indirect(&mut self, target: Reg) -> Result<()> {
        if target.is16() {
            bail!("expected 8-bit register, got '{target}'")
        }
        self.emit_byte(u8::from(LdRegIndirect))?;
        let source = self.expect_reg16()?;
        self.expect(&ParenClose)?;
        self.emit_byte(regs::u8_from(source, target))?;
        Ok(())
    }

    /// Generates a load register register instruction.
    ///
    /// # Errors
    ///
    /// * Mismatched register widths.
    pub fn gen_ld_reg_reg(&mut self, source: Reg, target: Reg) -> Result<()> {
        if source.is16() != target.is16() {
            bail!("expected same size register, got '{source}'")
        }
        self.emit_byte(u8::from(LdRegReg))?;
        self.emit_byte(regs::u8_from(source, target))
    }

    /// Generates a logical shift right instruction.
    ///
    /// # Errors
    ///
    /// * Syntax errors.
    pub fn gen_lsr(&mut self) -> Result<()> {
        let target = self.expect_reg8()?;
        self.emit_byte(u8::from(Lsr(target)))?;
        self.expect(&Comma)?;
        match self.next_token()? {
            ByteLiteral(shift) => self.emit_byte(shift),
            other => bail!("expected shift count, got {other}"),
        }
    }

    /// Generates a pop instruction.
    ///
    /// # Errors
    ///
    /// * Invalid register name.
    pub fn gen_pop(&mut self) -> Result<()> {
        let reg = self.expect_reg()?;
        self.emit_byte(u8::from(Pop(reg)))?;
        Ok(())
    }

    /// Generates a push instruction.
    ///
    /// # Errors
    ///
    /// * Invalid register name.
    pub fn gen_push(&mut self) -> Result<()> {
        let reg = self.expect_reg()?;
        self.emit_byte(u8::from(Push(reg)))?;
        Ok(())
    }

    /// Generates a store register direct instruction.
    ///
    /// # Errors
    ///
    /// * Missing comma before register name.
    /// * Invalid register name.
    pub fn gen_store_direct(&mut self, addr: u16) -> Result<()> {
        self.expect(&Comma)?;
        let reg = self.expect_reg8()?;
        self.emit_byte(u8::from(StoreRegDirect(reg)))?;
        self.emit_word(addr)?;
        Ok(())
    }

    /// Generates a store register indirect instruction.
    ///
    /// # Errors
    ///
    /// * Missing comma before register name.
    /// * Invalid register name.
    pub fn gen_store_indirect(&mut self) -> Result<()> {
        let target = self.expect_reg16()?;
        self.expect(&ParenClose)?;
        self.expect(&Comma)?;
        let source = self.expect_reg8()?;
        self.emit_byte(u8::from(StoreRegIndirect))?;
        self.emit_byte(regs::u8_from(source, target))?;
        Ok(())
    }

    /// Generates a subtract with carry instruction.
    ///
    /// # Errors
    ///
    /// * Missing register name.
    /// * Missing comma.
    /// * Missing or mis-sized operand.
    pub fn gen_sub(&mut self) -> Result<()> {
        let reg = self.expect_reg8()?;
        self.expect(&Comma)?;
        self.emit_byte(u8::from(Sub(reg)))?;
        let op = self.expect_op_for_reg(reg)?;
        for byte in op {
            self.emit_byte(byte)?;
        }
        Ok(())
    }

    /// Generates a trap instruction.
    ///
    /// # Errors
    ///
    /// * Invalid trap code.
    pub fn gen_trap(&mut self) -> Result<()> {
        let trap_code = match self.next_token()? {
            ByteLiteral(code @ 0x00..=0x3F) => code,
            ByteLiteral(code) => bail!("invalid trap code {code:#04X}"),
            other => bail!("expected trap code byte, got {other}"),
        };
        self.emit_byte(u8::from(Trap))?;
        self.emit_byte(trap_code)?;
        Ok(())
    }

    fn new(tokens: Vec<Token>) -> Self {
        Self {
            code: Vec::new(),
            cursor: 0,
            debug: false,
            labels: HashMap::new(),
            line: 1,
            loc: BASE,
            pass2: false,
            path: "input".to_owned(),
            tokens,
        }
    }

    /// Scans and returns the next token from the source code.
    ///
    /// # Errors
    ///
    /// * Unexpected end of input.
    pub fn next_token(&mut self) -> Result<Token> {
        let token = self
            .tokens
            .get(self.cursor)
            .ok_or(anyhow!("unexpected end of input"))?;
        self.cursor = self.cursor.strict_add(1);
        Ok(token.clone())
    }

    /// Processes an org directive.
    ///
    /// # Errors
    ///
    /// * Missing or invalid label or address.
    pub fn org(&mut self) -> Result<()> {
        let addr = match self.next_token()? {
            WordLiteral(addr) => addr,
            other => bail!("expected address, got {other}"),
        };
        if !self.code.is_empty() {
            let (offset, underflow) = addr.overflowing_sub(self.loc);
            if underflow {
                bail!("invalid address {addr} for 'org'")
            }
            self.code
                .extend(core::iter::repeat_n(0, usize::from(offset)));
        }
        self.loc = addr;
        Ok(())
    }

    /// Runs an assembly pass.
    ///
    /// # Errors
    ///
    /// * Syntax errors.
    pub fn pass(&mut self) -> Result<()> {
        while let Ok(token) = self.next_token() {
            match token {
                Comment(_) | Newline => {
                    self.line = self.line.checked_add(1).context("line count overflow")?;
                }
                Keyword(kw) => self.assemble_kw(&kw)?,
                LabelDef(label) => {
                    self.labels.insert(label, self.loc);
                }
                unexpected => bail!("unexpected token {unexpected}"),
            }
        }
        Ok(())
    }

    /// Returns the value associated with `label`.
    ///
    /// # Errors
    ///
    /// * Undefined label (on pass 2).
    pub fn resolve_label(&self, label: &str) -> Result<u16> {
        Ok(match self.labels.get(label) {
            Some(&addr) => addr,
            None if self.pass2 => bail!("undefined label {label}"),
            None => 0,
        })
    }
}

/// Disassembles a given binary program.
pub struct Disassembler<'code> {
    /// The code to be disassembled.
    pub code: Iter<'code, u8>,
}

impl<'code> From<&'code [u8]> for Disassembler<'code> {
    fn from(code: &'code [u8]) -> Self {
        Self { code: code.iter() }
    }
}

impl Iterator for Disassembler<'_> {
    type Item = String;

    /// Disassembles the next instruction.
    fn next(&mut self) -> Option<Self::Item> {
        let &opcode = self.code.next()?;
        Some(if let Ok(ins) = InstructionKind::try_from(opcode) {
            match ins {
                Add(reg) => format!("add {reg}, {}", self.format_byte()),
                And(reg) => format!("and {reg}, {}", self.format_byte()),
                BranchAlways => format!("bra {}", self.format_byte()),
                BranchCc => format!("bcc {}", self.format_byte()),
                BranchCs => format!("bcs {}", self.format_byte()),
                BranchEq => format!("beq {}", self.format_byte()),
                BranchNe => format!("bne {}", self.format_byte()),
                Call => format!("call {}", self.format_word()),
                Clc => "clc".into(),
                Cmp(reg) => format!("cmp {reg}, {}", self.format_op_for_reg(reg)),
                Dec(reg) => format!("dec {reg}"),
                DecIndirect => self.format_dec_indirect(),
                DecMem => format!("dec ({})", self.format_word()),
                Halt => "halt".into(),
                Inc(reg) => format!("inc {reg}"),
                IncIndirect => self.format_inc_indirect(),
                IncMem => format!("inc ({})", self.format_word()),
                Jmp => format!("jmp {}", self.format_word()),
                LdRegImm(reg) => format!("ld {reg}, {}", self.format_op_for_reg(reg)),
                LdRegIndirect => self.format_ld_reg_indirect(),
                LdRegReg => self.format_ld_reg_reg(),
                Lsr(reg) => format!("lsr {reg}, {}", self.format_byte()),
                Nop => "nop".into(),
                Pop(reg) => format!("pop {reg}"),
                Push(reg) => format!("push {reg}"),
                Ret => "ret".into(),
                Rti => "rti".into(),
                Sec => "sec".into(),
                StoreRegDirect(reg) => format!("ld {}, {reg}", self.format_word()),
                StoreRegIndirect => self.format_store_reg_indirect(),
                Sub(reg) => format!("sub {reg}, {}", self.format_byte()),
                Trap => format!("trap {}", self.format_byte()),
            }
        } else {
            format!("??? ({opcode:#04X})")
        })
    }
}

#[expect(clippy::elidable_lifetime_names, reason = "can't be elided here")]
impl<'code> Disassembler<'code> {
    /// Reads a byte operand and formats it for display.
    fn format_byte(&mut self) -> String {
        if let Some(op) = self.code.next() {
            format!("{op:#04X}")
        } else {
            "??? (no operand)".to_owned()
        }
    }

    /// Reads an operand specifying the source register and formats the
    /// instruction for display.
    fn format_dec_indirect(&mut self) -> String {
        if let Some(&regs) = self.code.next()
            && let Some(source) = source_from(regs)
            && source.is16()
        {
            format!("dec ({source})")
        } else {
            "??? (no operand)".to_owned()
        }
    }

    /// Reads an operand specifying the source register and formats the
    /// instruction for display.
    fn format_inc_indirect(&mut self) -> String {
        if let Some(&regs) = self.code.next()
            && let Some(source) = source_from(regs)
            && source.is16()
        {
            format!("inc ({source})")
        } else {
            "??? (no operand)".to_owned()
        }
    }

    /// Reads an operand specifying source and target registers and formats the
    /// instruction for display.
    fn format_ld_reg_indirect(&mut self) -> String {
        if let Some(&regs) = self.code.next()
            && let Some((source, target)) = source_and_target_from(regs)
        {
            format!("ld {target}, ({source})")
        } else {
            "??? (no operand)".to_owned()
        }
    }

    /// Reads an operand specifying source and target registers and formats the
    /// instruction for display.
    fn format_ld_reg_reg(&mut self) -> String {
        if let Some(&regs) = self.code.next()
            && let Some((source, target)) = source_and_target_from(regs)
        {
            format!("ld {target}, {source}")
        } else {
            "??? (no operand)".to_owned()
        }
    }

    /// Reads an operand for `reg` and formats it for display.
    fn format_op_for_reg(&mut self, reg: Reg) -> String {
        if reg.is16() {
            self.format_word()
        } else {
            self.format_byte()
        }
    }

    /// Reads an operand specifying source and target registers and formats the
    /// instruction for display.
    fn format_store_reg_indirect(&mut self) -> String {
        if let Some(&regs) = self.code.next()
            && let Some((source, target)) = source_and_target_from(regs)
        {
            format!("ld ({target}), {source}")
        } else {
            "??? (no operand)".to_owned()
        }
    }

    /// Reads a word operand and formats it for display.
    fn format_word(&mut self) -> String {
        if let (Some(&lo), Some(&hi)) = (self.code.next(), self.code.next()) {
            format!("{:#06X}", u16::from_le_bytes([lo, hi]))
        } else {
            "??? (no operand)".to_owned()
        }
    }
}

/// A source code token.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// Hexadecimal byte literal (`0x00`).
    ByteLiteral(u8),
    /// Comma.
    Comma,
    /// Comment.
    Comment(String),
    /// Identifier.
    Identifier(String),
    /// Illegal token.
    Illegal(String),
    /// Assembler keyword.
    Keyword(String),
    /// Label definition.
    LabelDef(String),
    /// Newline character (`\n`).
    Newline,
    /// Closing parenthesis (`)`).
    ParenClose,
    /// Opening parenthesis (`(`).
    ParenOpen,
    /// Register name.
    Register(Reg),
    /// String literal.
    StringLiteral(String),
    /// Hexadecimal word literal (`0x0000`).
    WordLiteral(u16),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            ByteLiteral(byte) => write!(f, "ByteLiteral({byte:#04X})"),
            WordLiteral(word) => write!(f, "WordLiteral({word:#06X})"),
            _ => Debug::fmt(self, f),
        }
    }
}

pub struct Tokenizer<'src> {
    /// Stores the source code being assembled.
    pub chars: Peekable<Chars<'src>>,
    /// Debug mode.
    pub debug: bool,
}

impl<'src> Tokenizer<'src> {
    /// Prints `msg` if in debug mode.
    pub fn debug_print(&self, msg: impl AsRef<str>) {
        if self.debug {
            println!("{}", msg.as_ref());
        }
    }

    #[must_use]
    pub fn new(source: &'src str) -> Self {
        Self {
            debug: false,
            chars: source.chars().peekable(),
        }
    }

    #[must_use]
    pub fn new_with_debug(source: &'src str) -> Self {
        let mut tokenizer = Self::new(source);
        tokenizer.debug = true;
        tokenizer
    }

    /// Reads a comment token.
    pub fn read_comment(&mut self) -> Token {
        self.chars.next(); // skip ';' prefix
        self.skip_whitespace();
        let comment: String =
            iter::from_fn(|| self.chars.next_if(|&ch| ch != '\r' && ch != '\n')).collect();
        self.chars.next_if(|&ch| ch == '\n'); // extra trailing newline on Windows
        Comment(comment)
    }

    /// Reads a hex literal token.
    pub fn read_hex_literal(&mut self) -> Token {
        self.chars.next();
        self.chars.next(); // skip "0x" prefix
        let literal: String =
            iter::from_fn(|| self.chars.next_if(char::is_ascii_hexdigit)).collect();
        match literal.len() {
            2 => match u8::from_str_radix(&literal, 16) {
                Ok(val) => ByteLiteral(val),
                Err(_) => Illegal(literal),
            },
            4 => match u16::from_str_radix(&literal, 16) {
                Ok(val) => WordLiteral(val),
                Err(_) => Illegal(literal),
            },
            _ => Illegal(literal),
        }
    }

    /// Reads an identifier, register name, or keyword.
    pub fn read_identifier(&mut self) -> Token {
        let ident: String = iter::from_fn(|| {
            self.chars
                .next_if(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
        .collect();
        self.debug_print(format!("ident: {ident}"));
        match ident.as_str() {
            _ if let Ok(reg) = Reg::from_str(&ident) => Register(reg),
            kw if KEYWORDS.contains(&kw) => Keyword(ident),
            label if let Some(&':') = self.chars.peek() => {
                self.chars.next();
                LabelDef(label.to_owned())
            }
            _ => Identifier(ident),
        }
    }

    /// Reads a string literal token.
    pub fn read_string(&mut self) -> Token {
        self.chars.next(); // skip opening " delimiter
        let string: String = iter::from_fn(|| self.chars.next_if(|&ch| ch != '"')).collect();
        self.chars.next(); // skip closing " delimiter
        StringLiteral(string)
    }

    /// Reads a given token.
    pub fn read_token(&mut self, token: Token) -> Token {
        self.chars.next();
        token
    }

    /// Advances to the next non-whitespace, non-newline character.
    pub fn skip_whitespace(&mut self) {
        while self
            .chars
            .next_if(|&ch| ch.is_whitespace() && ch != '\n')
            .is_some()
        {}
    }

    /// Scans tokens from the source code.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        self.skip_whitespace();
        while let Some(next_char) = self.chars.peek() {
            let next = *next_char;
            let token = match next {
                '"' => self.read_string(),
                '(' => self.read_token(ParenOpen),
                ')' => self.read_token(ParenClose),
                ',' => self.read_token(Comma),
                '0' => self.read_hex_literal(),
                ';' => self.read_comment(),
                '\n' => self.read_token(Newline),
                ch if ch.is_alphabetic() => self.read_identifier(),
                ch => self.read_token(Illegal(ch.to_string())),
            };
            self.debug_print(format!("token: {token}"));
            tokens.push(token);
            self.skip_whitespace();
        }
        tokens
    }
}

/// Formats the `data` slice as comma-separated hex bytes.
#[must_use]
pub fn as_hex(data: &[u8]) -> String {
    let mut byte_strs = Vec::new();
    for byte in data {
        byte_strs.push(format!("{byte:#04X}"));
    }
    format!("[{}]", byte_strs.join(", "))
}

/// Assembles `source`.
///
/// # Errors
///
/// * Syntax errors.
pub fn assemble(source: &str) -> Result<Vec<u8>> {
    let tokens = Tokenizer::new(source).tokenize();
    Assembler::new(tokens).assemble()
}

/// Assembles `source` with debug output.
///
/// # Errors
///
/// * Syntax errors.
pub fn assemble_with_debug(source: &str) -> Result<Vec<u8>> {
    let tokens = Tokenizer::new_with_debug(source).tokenize();
    let mut asm = Assembler::new(tokens);
    asm.debug = true;
    asm.assemble()
}

/// Assembles a program from the file at `path`.
///
/// # Errors
///
/// * File read errors
/// * Syntax errors
pub fn assemble_source_file(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let source = fs::read_to_string(&path)?;
    let tokens = Tokenizer::new(&source).tokenize();
    let mut asm = Assembler::new(tokens);
    asm.path = path.as_ref().display().to_string();
    asm.assemble()
}

/// Disassembles a single instruction from `code`.
///
/// Useful for writing tests.
#[must_use]
pub fn disassemble(code: &[u8]) -> String {
    let mut dis = Disassembler::from(code);
    dis.next().unwrap_or_default()
}

/// Tokenize `input`.
#[must_use]
pub fn tokenize(input: &str) -> Vec<Token> {
    Tokenizer::new(input).tokenize()
}

/// Tokenize `input` with verbose debugging.
#[must_use]
pub fn tokenize_with_debug(input: &str) -> Vec<Token> {
    Tokenizer::new_with_debug(input).tokenize()
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests")]
#[expect(clippy::expect_used, reason = "tests")]
#[expect(clippy::default_numeric_fallback, reason = "hex literals")]
mod tests {
    use super::*;

    macro_rules! assert_asm {
        ( $source:expr, $generated:expr, $object:expr ) => {
            assert_eq!(
                &$generated,
                $object,
                "wrong assembly for '{}': want {}, got {}",
                $source,
                as_hex($object),
                as_hex(&$generated),
            );
        };
    }

    macro_rules! assert_disasm {
        ( $generated:expr, $source:expr ) => {
            assert_eq!(
                &disassemble(&$generated),
                $source,
                "wrong disassembly for {}",
                as_hex(&$generated)
            );
        };
    }

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
    fn assembler_accepts_label_as_immediate_word_value() {
        let source = "
        LABEL:
            ld cd, LABEL
";
        let generated = assemble_with_debug(source).unwrap();
        assert_asm!(
            source,
            generated,
            &[u8::from(LdRegImm(Reg::CD)), 0x00, 0x01]
        );
    }

    #[test]
    fn assembler_counts_lines_correctly() {
        let source = "ld a, 0xFF\ninc a\nhalt";
        let mut asm = Assembler::new(tokenize(source));
        asm.debug = true;
        asm.assemble().unwrap();
        assert_eq!(asm.line, 3, "wrong line count");
    }

    #[test]
    fn assembler_uses_0x0100_as_default_base_address() {
        let source = "
        call AHEAD
        halt
    AHEAD:
        halt
";
        let mut asm = Assembler::new(tokenize(source));
        asm.debug = true;
        asm.assemble().unwrap();
        assert_hex!(asm.resolve_label("AHEAD").unwrap(), 0x0104, "wrong address");
    }

    #[test]
    fn assembler_ignores_comments() {
        let source = "ld a, 0xFF ; loop count";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[u8::from(LdRegImm(Reg::A)), 0xFF];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn assembler_resolves_backward_labels() {
        let source = "
        ld a, 0x06 ; about 1 second
    LOOP:
        ld cd, 0xFFFF ; inner loop
    INNER_LOOP:
        dec cd
        bne INNER_LOOP
        dec a
        bne LOOP
        halt
";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[
            u8::from(LdRegImm(Reg::A)),
            0x06,
            u8::from(LdRegImm(Reg::CD)),
            0xFF,
            0xFF,
            u8::from(Dec(Reg::CD)),
            u8::from(BranchNe),
            0xFD,
            u8::from(Dec(Reg::A)),
            u8::from(BranchNe),
            0xF7,
            u8::from(Halt),
        ];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn assembler_resolves_forward_labels_for_branches() {
        let source = "
        bra AHEAD
        halt
    AHEAD:
        halt
";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[u8::from(BranchAlways), 0x01, u8::from(Halt), u8::from(Halt)];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn assembler_resolves_forward_labels_for_calls() {
        let source = "
        call AHEAD
        halt
    AHEAD:
        halt
";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[u8::from(Call), 0x04, 0x01, u8::from(Halt), u8::from(Halt)];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn data_emits_literal_bytes() {
        let source = "
        nop
        data 0x01, 0x02, 0x03
        halt
";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[u8::from(Nop), 0x01, 0x02, 0x03, u8::from(Halt)];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn data_emits_bytes_for_strings() {
        let source = "
        nop
        data 0x01, \"hello\", 0x0A, 0x03
        halt
";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[
            u8::from(Nop),
            0x01,
            0x68,
            0x65,
            0x6C,
            0x6C,
            0x6F,
            0x0A,
            0x03,
            u8::from(Halt),
        ];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn emit_byte_fn_detects_wraparound_of_memory() {
        let source = "
        org 0xFFFF
        data 0x01, 0x02";
        assemble_with_debug(source).expect_err("overflowing memory should fail");
    }

    #[test]
    fn emit_word_fn_detects_wraparound_of_memory_before_word() {
        let source = "
        org 0xFFFF
        call 0xBABE";
        assemble_with_debug(source).expect_err("overflowing memory should fail");
    }

    #[test]
    fn emit_word_fn_detects_wraparound_of_memory_in_mid_word() {
        let source = "
        org 0xFFFE
        call 0xBABE";
        assemble_with_debug(source).expect_err("overflowing memory should fail");
    }

    #[test]
    fn org_adjusts_subsequent_label_address() {
        let source = "
        org 0xC000
        call AHEAD
        halt
    AHEAD:
        halt
";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[u8::from(Call), 0x04, 0xC0, u8::from(Halt), u8::from(Halt)];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn org_after_code_start_pads_with_zeroes() {
        let source = "
        ld a, 0xFF
        bra AHEAD
        org 0x0108
    AHEAD:
        halt
";
        let generated = assemble_with_debug(source).unwrap();
        let object = &[
            u8::from(LdRegImm(Reg::A)),
            0xFF,
            u8::from(BranchAlways),
            0x04,
            0x00,
            0x00,
            0x00,
            0x00,
            u8::from(Halt),
        ];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn get_displacement_fn_calculates_correct_max_displacements() {
        let mut source = String::from("LOOP:\n");
        source.push_str("nop\n".repeat(126).as_str());
        source.push_str("beq LOOP");
        let generated = assemble_with_debug(&source).unwrap();
        let mut object = vec![u8::from(Nop); 126];
        object.extend([u8::from(BranchEq), 0x80]);
        assert_asm!(source, generated, &object);
    }

    #[test]
    fn get_displacement_fn_rejects_out_of_range_displacement() {
        let mut source = String::from("LOOP:\n");
        source.push_str("nop\n".repeat(127).as_str());
        source.push_str("beq LOOP");
        assemble_with_debug(source.as_str()).expect_err("invalid displacement should be rejected");
    }

    #[test]
    fn disassembler_correctly_disassembles_multiline_programs() {
        let source = "ld a, 0x01\ndec a\nld b, 0x02\ninc b\nld c, 0x03\ndec c\ndec c";
        let code = assemble_with_debug(source).unwrap();
        let output: Vec<_> = Disassembler::from(code.as_slice()).collect();
        assert_eq!(output.join("\n"), source);
    }

    #[test]
    fn disassembler_copes_with_invalid_code() {
        assert_disasm!([0x10], "ld a, ??? (no operand)");
        assert_disasm!([0x1D], "??? (no operand)");
        assert_disasm!([0x28], "??? (no operand)");
        assert_disasm!([0x1F], "??? (no operand)");
        assert_disasm!([0x3D], "??? (no operand)");
        assert_disasm!([0x3E], "inc (??? (no operand))");
        assert_disasm!([0x4D], "??? (no operand)");
        assert_disasm!([0x4E], "dec (??? (no operand))");
        // Invalid opcode
        assert_disasm!([0xFF, 0xFF], "??? (0xFF)");
    }

    #[test]
    fn as_hex_fn_formats_value_as_hex() {
        assert_eq!(as_hex(&[1, 255]), "[0x01, 0xFF]");
    }

    #[test]
    fn assembler_assembles_and_disassembles_instructions_correctly() {
        use Reg::*;
        let cases: &[(&str, &[u8])] = &[
            ("", &[]),
            ("add a, 0x01", &[u8::from(Add(A)), 0x01]),
            ("and a, 0x01", &[u8::from(And(A)), 0x01]),
            ("bcc 0x10", &[u8::from(BranchCc), 0x10]),
            ("bcs 0x10", &[u8::from(BranchCs), 0x10]),
            ("beq 0xF0", &[u8::from(BranchEq), 0xF0]),
            ("bne 0x01", &[u8::from(BranchNe), 0x01]),
            ("bra 0x99", &[u8::from(BranchAlways), 0x99]),
            ("call 0xBEEE", &[u8::from(Call), 0xEE, 0xBE]),
            ("clc", &[u8::from(Clc)]),
            ("cmp d, 0x01", &[u8::from(Cmp(D)), 0x01]),
            ("cmp gh, 0xDEAD", &[u8::from(Cmp(GH)), 0xAD, 0xDE]),
            ("dec (0xBABE)", &[u8::from(DecMem), 0xBE, 0xBA]),
            ("dec (gh)", &[u8::from(DecIndirect), 0xB0]),
            ("dec ab", &[u8::from(Dec(AB))]),
            ("dec g", &[u8::from(Dec(G))]),
            ("halt", &[u8::from(Halt)]),
            ("inc (0xCAFE)", &[u8::from(IncMem), 0xFE, 0xCA]),
            ("inc (cd)", &[u8::from(IncIndirect), 0x90]),
            ("inc a", &[u8::from(Inc(A))]),
            ("inc ef", &[u8::from(Inc(EF))]),
            ("jmp 0x1234", &[u8::from(Jmp), 0x34, 0x12]),
            ("ld (ef), a", &[u8::from(StoreRegIndirect), 0x0A]),
            ("ld 0x00AF, h", &[u8::from(StoreRegDirect(H)), 0xAF, 0x00]),
            ("ld a, b", &[u8::from(LdRegReg), 0x10]),
            ("ld ab, 0x000F", &[u8::from(LdRegImm(AB)), 0x0F, 0x00]),
            ("ld b, (cd)", &[u8::from(LdRegIndirect), 0x91]),
            ("ld b, 0xFF", &[u8::from(LdRegImm(B)), 0xFF]),
            ("ld cd, 0xBEEF", &[u8::from(LdRegImm(CD)), 0xEF, 0xBE]),
            ("ld sp, 0x010F", &[u8::from(LdRegImm(SP)), 0x0F, 0x01]),
            ("lsr a, 0x04", &[u8::from(Lsr(A)), 0x04]),
            ("nop", &[u8::from(Nop)]),
            ("pop e", &[u8::from(Pop(E))]),
            ("push c", &[u8::from(Push(C))]),
            ("ret", &[u8::from(Ret)]),
            ("rti", &[u8::from(Rti)]),
            ("sec", &[u8::from(Sec)]),
            ("sub a, 0x01", &[u8::from(Sub(A)), 0x01]),
            ("trap 0x01", &[u8::from(Trap), 0x01]),
        ];
        for &(source, object) in cases {
            let generated = assemble_with_debug(source).unwrap();
            assert_asm!(source, generated, object);
            assert_disasm!(generated, source);
        }
    }

    #[test]
    #[expect(clippy::non_ascii_literal, reason = "test")]
    fn assembler_rejects_invalid_code() {
        let cases: &[&str] = &[
            "&",
            "add",
            "add a",
            "add a, cd",
            "add ab, 0xFFFF",
            "and",
            "and ab, 0xFF",
            "and cd, 0xC0DE",
            "and a, 0x1000",
            "and 0x01, 0x0F",
            "bcc",
            "bcs a",
            "bcs 0x1000",
            "beq UNDEFINED_LABEL",
            "beq",
            "bne 0x1000",
            "bogus",
            "bra a",
            "call 0x01",
            "call gh",
            "call",
            "clc 0x00",
            "cmp a, b, d",
            "data (",
            "data \"©\"",
            "data",
            "dec (",
            "dec (a)",
            "dec z",
            "dec",
            "inc (",
            "inc (0xFF)",
            "inc (a)",
            "inc ,",
            "inc 0x0000",
            "inc ax",
            "inc",
            "jmp",
            "jmp ab",
            "jmp 0x01",
            "ld (0x0), b",
            "ld 0x0000",
            "ld 0x0000, ",
            "ld 0x00AF, ab",
            "ld a 0x01",
            "ld a",
            "ld a, ",
            "ld a, (bogus)",
            "ld a, 0x1000",
            "ld a, 0xZ",
            "ld a, LABEL",
            "ld a, cd",
            "ld ab, (cd)",
            "ld ab, 0x--",
            "ld ab, 0x00009",
            "ld ab, UNDEFINED",
            "ld bogus, 0x01",
            "ld cd, 0x09",
            "ld ef",
            "ld ef, 0x02",
            "ld",
            "lsr",
            "lsr a",
            "lsr ab, 0x02",
            "nop\norg 0x0000",
            "org 0xFFFF\nld a, 0x01",
            "pop 0x01",
            "push",
            "ret cd",
            "rti 0x0100",
            "sec ab",
            "sub",
            "sub a",
            "sub a, cd",
            "sub ab, 0xFFFF",
            "trap 0x40",
            "trap 0xFF",
            "trap a",
            "trap",
        ];
        for &source in cases {
            assemble_with_debug(source)
                .expect_err(&format!("assembling invalid source '{source}' should fail"));
        }
    }
}
