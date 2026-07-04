use anyhow::{Context as _, Result, bail};

use core::{
    fmt::{Debug, Display, Formatter},
    iter::{self, Peekable},
    slice::Iter,
    str::{Chars, FromStr as _},
};
use std::collections::HashMap;

use crate::instructions::InstructionKind;
use crate::{instructions::InstructionKind::*, regs::Reg};

/// Keywords recognised by the assembler.
pub const KEYWORDS: &[&str] = &["beq", "bne", "dec", "halt", "inc", "ld", "nop"];

/// Assembles a given program.
#[non_exhaustive]
pub struct Assembler<'src> {
    /// Stores the source code being assembled.
    pub chars: Peekable<Chars<'src>>,
    /// Generated object code.
    pub code: Vec<u8>,
    /// Enables verbose debugging.
    pub debug: bool,
    /// Label table.
    pub labels: HashMap<String, u16>,
}

impl<'src> From<&'src str> for Assembler<'src> {
    #[inline]
    fn from(source: &'src str) -> Self {
        Self {
            chars: source.chars().peekable(),
            code: Vec::new(),
            debug: false,
            labels: HashMap::new(),
        }
    }
}

impl Assembler<'_> {
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "any unexpected token is illegal here"
    )]
    /// Assembles the source code.
    ///
    /// # Errors
    ///
    /// * Unexpected end of file
    /// * Invalid register name
    /// * Unexpected token
    #[inline]
    pub fn assemble(&mut self) -> Result<Vec<u8>> {
        while let Some(token) = self.next_token() {
            match token {
                Token::Comment(_) => {}
                Token::Keyword(kw) if kw == "beq" => {
                    let dis = self.get_displacement()?;
                    self.code.extend([u8::from(BranchEq), dis]);
                }
                Token::Keyword(kw) if kw == "bne" => {
                    let dis = self.get_displacement()?;
                    self.code.extend([u8::from(BranchNe), dis]);
                }
                Token::Keyword(kw) if kw == "dec" => {
                    let Some(Token::Register(reg)) = self.next_token() else {
                        bail!("expected register name")
                    };
                    self.code.push(u8::from(Dec(reg)));
                }
                Token::Keyword(kw) if kw == "halt" => self.code.push(u8::from(Halt)),
                Token::Keyword(kw) if kw == "inc" => {
                    let Some(Token::Register(reg)) = self.next_token() else {
                        bail!("expected register name")
                    };
                    self.code.push(u8::from(Inc(reg)));
                }
                Token::Keyword(kw) if kw == "ld" => match self.next_token() {
                    Some(Token::WordLiteral(addr)) => self.gen_store_direct(addr)?,
                    Some(Token::Register(reg)) => self.gen_ld_imm(reg)?,
                    Some(other) => bail!("expected register name, got {other:?}"),
                    None => bail!("unexpected end of file"),
                },
                Token::Keyword(kw) if kw == "nop" => self.code.push(u8::from(Nop)),
                Token::LabelDef(label) => {
                    self.labels.insert(label, self.offset());
                }
                unexpected => bail!("unexpected token {unexpected:?}"),
            }
        }
        Ok(self.code.clone())
    }

    /// Skips the given string (or as much of it as is present in the source).
    #[inline]
    pub fn chomp(&mut self, st: &str) -> Option<()> {
        for want in st.chars() {
            _ = self.chars.next_if(|&got| want == got)?;
        }
        Some(())
    }

    /// Generate a load register immediate instruction.
    ///
    /// # Errors
    /// * Missing comma after register name
    /// * Missing or wrong size operand
    #[inline]
    pub fn gen_ld_imm(&mut self, reg: Reg) -> Result<()> {
        use crate::regs::Reg::*;
        self.code.push(u8::from(LoadRegImm(reg)));
        let Some(Token::Comma) = self.next_token() else {
            bail!("expected comma")
        };
        match reg {
            A | B | C | D | E | F | G | H => {
                let Some(Token::ByteLiteral(operand)) = self.next_token() else {
                    bail!("expected byte operand for 8-bit immediate 'ld {reg}'")
                };
                self.code.push(operand);
            }
            AB | CD | EF | GH => {
                let Some(Token::WordLiteral(operand)) = self.next_token() else {
                    bail!("expected word operand for 16-bit immediate 'ld {reg}'")
                };
                self.code.extend(operand.to_le_bytes());
            }
        }

        Ok(())
    }

    /// Generate a store register direct instruction.
    ///
    /// # Errors
    ///
    /// * Missing comma before register name
    /// * Invalid register name
    #[inline]
    pub fn gen_store_direct(&mut self, addr: u16) -> Result<()> {
        let Some(Token::Comma) = self.next_token() else {
            bail!("expected comma")
        };
        let Some(Token::Register(reg)) = self.next_token() else {
            bail!("expected register name")
        };
        self.code.push(u8::from(StoreRegDirect(reg)));
        self.code.extend(addr.to_le_bytes());
        Ok(())
    }

    /// Returns the displacement for the current branch instruction.
    ///
    /// This may be an immediate value or a label reference.
    ///
    /// # Errors
    ///
    /// * Undefined label
    /// * No displacement
    /// * Displacement out of range (signed byte)
    #[expect(clippy::as_conversions, reason = "required for encoding")]
    #[expect(clippy::cast_possible_truncation, reason = "code ensures valid range")]
    #[inline]
    pub fn get_displacement(&mut self) -> Result<u8> {
        match self.next_token() {
            Some(Token::Identifier(label)) => {
                let Some(&addr) = self.labels.get(&label) else {
                    bail!("undefined label {label}")
                };
                // Displacement is calculated relative to the address of the next
                // instruction (i.e. PC+2)
                let long_dis = addr.wrapping_sub(self.offset()).wrapping_sub(2);
                // This must fit into a signed byte (-128 <= dis <= +127)
                match long_dis {
                    0x0000..=0x007F | 0xFF80..=0xFFFF => Ok(long_dis as u8),
                    _ => bail!("displacement out of range: {long_dis:#06X}"),
                }
            }
            Some(Token::ByteLiteral(dis)) => Ok(dis),
            _ => bail!("expected displacement"),
        }
    }

    /// Scans and returns the next token from the source code.
    #[inline]
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if let Some(next_char) = self.chars.peek() {
            let next = *next_char;
            let token = match next {
                '0' => self.read_hex_literal(),
                ',' => self.read_token(Token::Comma),
                ';' => self.read_comment(),
                ch if ch.is_alphabetic() => self.read_identifier(),
                ch => self.read_illegal(ch),
            };
            if self.debug {
                println!("token: {token}");
            }
            Some(token)
        } else {
            None
        }
    }

    /// Returns the current offset (PC) position in the generated code.
    #[expect(clippy::as_conversions, reason = "max code size = 65536")]
    #[expect(clippy::cast_possible_truncation, reason = "max code size = 65536")]
    #[inline]
    #[must_use]
    pub fn offset(&self) -> u16 {
        self.code.len() as u16
    }

    /// Reads a comment token.
    #[inline]
    pub fn read_comment(&mut self) -> Token {
        self.chars.next();
        self.skip_whitespace();
        let comment: String =
            iter::from_fn(|| self.chars.next_if(|&ch| ch != '\r' && ch != '\n')).collect();
        self.chars.next_if(|&ch| ch == '\n'); // extra trailing newline on Windows
        Token::Comment(comment)
    }

    /// Reads a hex literal token.
    #[inline]
    pub fn read_hex_literal(&mut self) -> Token {
        self.chomp("0x");
        let literal: String =
            iter::from_fn(|| self.chars.next_if(char::is_ascii_hexdigit)).collect();
        match literal.len() {
            4 => match u16::from_str_radix(&literal, 16) {
                Ok(val) => Token::WordLiteral(val),
                Err(_) => Token::Illegal(literal),
            },
            2 => match u8::from_str_radix(&literal, 16) {
                Ok(val) => Token::ByteLiteral(val),
                Err(_) => Token::Illegal(literal),
            },
            _ => Token::Illegal(literal),
        }
    }

    /// Reads an identifier, register name, or keyword.
    #[inline]
    pub fn read_identifier(&mut self) -> Token {
        let ident: String = iter::from_fn(|| self.chars.next_if(|ch| ch.is_alphabetic())).collect();
        if self.debug {
            println!("ident: {ident}");
        }
        match ident.as_str() {
            _ if let Ok(reg) = Reg::from_str(&ident) => Token::Register(reg),
            kw if KEYWORDS.contains(&kw) => Token::Keyword(ident),
            label if let Some(&':') = self.chars.peek() => {
                self.chars.next();
                Token::LabelDef(label.to_owned())
            }
            _ => Token::Identifier(ident),
        }
    }

    /// Reads an illegal token.
    #[inline]
    pub fn read_illegal(&mut self, ch: char) -> Token {
        self.chars.next();
        Token::Illegal(ch.to_string())
    }

    /// Reads a given token.
    #[inline]
    pub fn read_token(&mut self, token: Token) -> Token {
        self.chars.next();
        token
    }

    /// Advances to the next non-whitespace character.
    #[inline]
    pub fn skip_whitespace(&mut self) {
        while self.chars.next_if(|ch| ch.is_whitespace()).is_some() {}
    }
}

#[non_exhaustive]
pub struct Disassembler<'code> {
    pub code: Iter<'code, u8>,
}

impl<'code> From<&'code [u8]> for Disassembler<'code> {
    #[inline]
    fn from(code: &'code [u8]) -> Self {
        Self { code: code.iter() }
    }
}

impl Iterator for Disassembler<'_> {
    type Item = String;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        use crate::regs::Reg::*;
        let &opcode = self.code.next()?;
        Some(if let Ok(ins) = InstructionKind::try_from(opcode) {
            match ins {
                BranchEq => format!("beq {}", format_maybe_byte(self.code.next())),
                BranchNe => format!("bne {}", format_maybe_byte(self.code.next())),
                Dec(reg) => format!("dec {reg}"),
                Halt => "halt".into(),
                Inc(reg) => format!("inc {reg}"),
                Nop => "nop".into(),
                LoadRegImm(reg) => {
                    let op_lo = self.code.next();
                    format!(
                        "ld {reg}, {}",
                        match reg {
                            A | B | C | D | E | F | G | H => format_maybe_byte(op_lo),
                            AB | CD | EF | GH => {
                                let op_hi = self.code.next();
                                format_maybe_word(op_lo, op_hi)
                            }
                        }
                    )
                }
                StoreRegDirect(reg) => {
                    let op_lo = self.code.next();
                    let op_hi = self.code.next();
                    format!("ld {}, {reg}", format_maybe_word(op_lo, op_hi))
                }
            }
        } else {
            format!("??? ({opcode:#04X})")
        })
    }
}

/// A source code token.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Token {
    ByteLiteral(u8),
    Comma,
    Comment(String),
    Identifier(String),
    Illegal(String),
    Keyword(String),
    LabelDef(String),
    Register(Reg),
    RegisterName(String),
    WordLiteral(u16),
}

impl Display for Token {
    #[expect(clippy::wildcard_enum_match_arm, reason = "debug formatting is okay")]
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            Token::ByteLiteral(byte) => write!(f, "ByteLiteral({byte:#04X})"),
            Token::WordLiteral(word) => write!(f, "WordLiteral({word:#06X})"),
            _ => Debug::fmt(self, f),
        }
    }
}

/// Assembles `source` with debug outupt, panicking on any error.
///
/// Useful for writing tests.
///
/// # Panics
///
/// If the program fails to assemble.
#[expect(clippy::unwrap_used, reason = "for testing")]
#[inline]
#[must_use]
pub fn asm(source: &str) -> Vec<u8> {
    let mut asm = Assembler::from(source);
    asm.debug = true;
    asm.assemble()
        .context(format!("assembling '{source}'"))
        .unwrap()
}

/// Disassembles a single instruction from `code`.
///
/// Useful for writing tests.
#[inline]
#[must_use]
pub fn disassemble(code: &[u8]) -> Option<String> {
    let mut dis = Disassembler::from(code);
    dis.next()
}

fn format_maybe_byte(maybe_op: Option<&u8>) -> String {
    if let Some(op) = maybe_op {
        format!("{op:#04X}")
    } else {
        "??? (no operand)".to_owned()
    }
}

fn format_maybe_word(maybe_lo: Option<&u8>, maybe_hi: Option<&u8>) -> String {
    if let (Some(&lo), Some(&hi)) = (maybe_lo, maybe_hi) {
        format!("{:#06X}", u16::from_le_bytes([lo, hi]))
    } else {
        "??? (no operand)".to_owned()
    }
}

#[expect(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
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
                &disassemble(&$generated).unwrap(),
                $source,
                "wrong disassembly for {}",
                as_hex(&$generated)
            );
        };
    }

    #[test]
    fn assembler_assembles_and_disassembles_instructions_correctly() {
        use Reg::*;
        let cases: &[(&str, &[u8])] = &[
            ("beq 0xF0", &[u8::from(BranchEq), 0xF0]),
            ("bne 0x01", &[u8::from(BranchNe), 0x01]),
            ("dec g", &[u8::from(Dec(G))]),
            ("halt", &[u8::from(Halt)]),
            ("inc a", &[u8::from(Inc(A))]),
            ("ld b, 0xFF", &[u8::from(LoadRegImm(B)), 0xFF]),
            ("ld cd, 0xBEEF", &[u8::from(LoadRegImm(CD)), 0xEF, 0xBE]),
            ("ld 0x00AF, h", &[u8::from(StoreRegDirect(H)), 0xAF, 0x00]),
            ("nop", &[u8::from(Nop)]),
        ];
        for &(source, object) in cases {
            let generated = asm(source);
            assert_asm!(source, generated, object);
            assert_disasm!(generated, source);
        }
    }

    #[test]
    fn assembler_ignores_comments() {
        let source = "ld a, 0xFF ; loop count";
        let generated = asm(source);
        let object = &[u8::from(LoadRegImm(Reg::A)), 0xFF];
        assert_asm!(source, generated, object);
    }

    #[test]
    fn assembler_resolves_backward_labels() {
        let source = "
        ld a, 0x06 ; about 1 second
    LOOP:
        ld cd, 0xFFFF ; inner loop
    INNER:
        dec cd
        bne INNER
        dec a
        bne LOOP
        halt
";
        let generated = asm(source);
        let object = &[
            u8::from(LoadRegImm(Reg::A)),
            0x06,
            u8::from(LoadRegImm(Reg::CD)),
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
    fn get_displacement_fn_calculates_correct_max_displacements() {
        let mut source = String::from("LOOP:\n");
        source.push_str("nop\n".repeat(126).as_str());
        source.push_str("beq LOOP");
        let generated = asm(&source);
        let mut object = vec![u8::from(Nop); 126];
        object.extend([u8::from(BranchEq), 0x80]);
        assert_asm!(source, generated, &object);
    }

    #[expect(clippy::expect_used, reason = "test")]
    #[test]
    fn get_displacement_fn_rejects_out_of_range_displacement() {
        let mut source = String::from("LOOP:\n");
        source.push_str("nop\n".repeat(127).as_str());
        source.push_str("beq LOOP");
        let mut asm = Assembler::from(source.as_str());
        asm.assemble()
            .expect_err("invalid displacement should be rejected");
    }

    #[test]
    fn disassembler_correctly_disassembles_multiline_programs() {
        let source = "ld a, 0x01
dec a
ld b, 0x02
inc b
ld c, 0x03
dec c
dec c";
        let code = Assembler::from(source).assemble().unwrap();
        let output: Vec<_> = Disassembler::from(code.as_slice()).collect();
        assert_eq!(output.join("\n"), source);
    }

    #[test]
    fn disassembler_copes_with_invalid_code() {
        // Load immediate without a following operand
        assert_disasm!([0x10], "ld a, ??? (no operand)");
        // Invalid opcode
        assert_disasm!([0x1C, 0xFF], "??? (0x1C)");
    }

    fn as_hex(data: &[u8]) -> String {
        let mut byte_strs = Vec::new();
        for byte in data {
            byte_strs.push(format!("{byte:#04X}"));
        }
        format!("[{}]", byte_strs.join(", "))
    }
}
