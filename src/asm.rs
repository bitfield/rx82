use anyhow::{Result, bail};

use core::fmt::{Debug, Display};
use core::{
    fmt::Formatter,
    iter::{self, Peekable},
    str::Chars,
    str::FromStr as _,
};

use crate::{
    instructions::Instruction,
    regs::{Reg8, Reg16},
};

/// Keywords recognised by the assembler.
pub const KEYWORDS: &[&str] = &["halt", "ld", "nop"];

/// 8-bit register names.
pub const REG8: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h"];

/// 16-bit register pair names.
pub const REG16: &[&str] = &["ab", "cd", "ef", "gh"];

/// Assembles a given program.
#[non_exhaustive]
pub struct Assembler<'src> {
    /// Stores the source code being assembled.
    pub chars: Peekable<Chars<'src>>,
    /// Generated object code.
    pub code: Vec<u8>,
    /// Enables verbose debugging.
    pub debug: bool,
}

impl<'src> From<&'src str> for Assembler<'src> {
    #[inline]
    fn from(source: &'src str) -> Self {
        Self {
            chars: source.chars().peekable(),
            code: Vec::new(),
            debug: false,
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
                Token::Keyword(kw) if kw == "halt" => self.code.push(u8::from(Instruction::Halt)),
                Token::Keyword(kw) if kw == "nop" => self.code.push(u8::from(Instruction::Nop)),
                Token::Keyword(kw) if kw == "ld" => match self.next_token() {
                    // Some(Token::ParenOpen) => self.gen_ld_mem8()?,
                    Some(Token::Register8(reg)) => self.gen_ld_imm8(reg)?,
                    Some(Token::Register16(reg)) => self.gen_ld_imm16(reg)?,
                    Some(other) => bail!("expected register name, got {other:?}"),
                    None => bail!("unexpected end of file"),
                },
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

    /// Assembles the source code, with debugging output.
    ///
    /// # Errors
    ///
    /// If the source is invalid.
    #[inline]
    pub fn debug_assemble(&mut self) -> Result<Vec<u8>> {
        self.debug = true;
        self.assemble()
    }

    /// Prints a message if debug mode is on.
    #[inline]
    pub fn debug_token(&self, token: &Token) {
        if self.debug {
            println!("token: {token}");
        }
    }

    /// Generate a 16-bit load register immediate instruction.
    ///
    /// # Errors
    ///
    /// * Missing comma after register name
    /// * Missing word literal operand
    #[inline]
    pub fn gen_ld_imm16(&mut self, reg: Reg16) -> Result<()> {
        self.code.push(u8::from(Instruction::LoadRegImm16(reg)));
        let Some(Token::Comma) = self.next_token() else {
            bail!("expected comma")
        };
        let Some(Token::WordLiteral(operand)) = self.next_token() else {
            bail!("expected word operand for 16-bit immediate 'ld {reg}'")
        };
        self.code.extend(operand.to_le_bytes());
        Ok(())
    }

    /// Generate an 8-bit load register immediate instruction.
    ///
    /// # Errors
    /// * Missing comma after register name
    /// * Missing byte literal operand
    #[inline]
    pub fn gen_ld_imm8(&mut self, reg: Reg8) -> Result<()> {
        self.code.push(u8::from(Instruction::LoadRegImm8(reg)));
        let Some(Token::Comma) = self.next_token() else {
            bail!("expected comma")
        };
        let Some(Token::ByteLiteral(operand)) = self.next_token() else {
            bail!("expected byte operand for 8-bit immediate 'ld {reg}'")
        };
        self.code.push(operand);
        Ok(())
    }

    // Generate an 8-bit load memory from register instruction.

    // # Errors

    // * Missing address operand
    // * Missing closing parenthesis after address
    // * Missing comma before register name
    // * Invalid register name
    // #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
    // #[inline]
    // pub fn gen_ld_mem8(&mut self) -> Result<()> {
    //     let Token::WordLiteral(addr) = self.read_hex_literal_addr() else {
    //         bail!("expected address")
    //     };
    //     self.debug_token(&Token::WordLiteral(addr));
    //     let Some(Token::ParenClose) = self.next_token() else {
    //         bail!("expected closing parenthesis")
    //     };
    //     let Some(Token::Comma) = self.next_token() else {
    //         bail!("expected comma")
    //     };
    //     let Some(Token::Register8(reg)) = self.next_token() else {
    //         bail!("expected register name")
    //     };
    //     let opcode = match reg {
    //         Reg8::A => LdMemByteA,
    //         Reg8::B => LdMemByteB,
    //         Reg8::C => LdMemByteC,
    //         Reg8::D => LdMemByteD,
    //         Reg8::E => LdMemByteE,
    //         Reg8::F => LdMemByteF,
    //         Reg8::G => LdMemByteG,
    //         Reg8::H => LdMemByteH,
    //     };
    //     self.code.push(opcode as u8);
    //     self.code.extend(addr.to_le_bytes());
    //     Ok(())
    // }

    /// Scans and returns the next token from the source code.
    #[inline]
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if let Some(next_char) = self.chars.peek() {
            let next = *next_char;
            let token = match next {
                '0' => self.read_hex_literal(),
                ',' => self.read_token(Token::Comma),
                '(' => self.read_token(Token::ParenOpen),
                ')' => self.read_token(Token::ParenClose),
                ch if ch.is_alphabetic() => self.read_identifier(),
                ch => self.read_illegal(ch),
            };
            self.debug_token(&token);
            Some(token)
        } else {
            None
        }
    }

    /// Reads a hex literal token.
    #[inline]
    pub fn read_hex_literal(&mut self) -> Token {
        self.chomp("0x");
        let literal: String =
            iter::from_fn(|| self.chars.next_if(char::is_ascii_hexdigit)).collect();
        match u8::from_str_radix(&literal, 16) {
            Ok(val) => Token::ByteLiteral(val),
            Err(_) => match u16::from_str_radix(&literal, 16) {
                Ok(val) => Token::WordLiteral(val),
                Err(_) => Token::Illegal(literal),
            },
        }
    }

    /// Reads a hex literal address token.
    #[inline]
    pub fn read_hex_literal_addr(&mut self) -> Token {
        self.chomp("0x");
        let literal: String =
            iter::from_fn(|| self.chars.next_if(char::is_ascii_hexdigit)).collect();
        match u16::from_str_radix(&literal, 16) {
            Ok(val) => Token::WordLiteral(val),
            Err(_) => Token::Illegal(literal),
        }
    }

    /// Reads an identifier, register name, or keyword.
    #[inline]
    pub fn read_identifier(&mut self) -> Token {
        let ident: String = iter::from_fn(|| self.chars.next_if(|ch| ch.is_alphabetic())).collect();
        match ident.as_str() {
            _ if let Ok(reg) = Reg8::from_str(&ident) => Token::Register8(reg),
            _ if let Ok(reg) = Reg16::from_str(&ident) => Token::Register16(reg),
            kw if KEYWORDS.contains(&kw) => Token::Keyword(ident),
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

/// A source code token.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Token {
    ByteLiteral(u8),
    Comma,
    Identifier(String),
    Illegal(String),
    Keyword(String),
    ParenClose,
    ParenOpen,
    Register16(Reg16),
    Register8(Reg8),
    RegisterName(String),
    WordLiteral(u16),
}

impl Display for Token {
    #[expect(clippy::absolute_paths, reason = "disambiguate from anyhow::Result")]
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

/// Disassembles a single instruction from `slice`.
#[inline]
#[must_use]
pub fn disassemble(slice: &[u8]) -> String {
    use crate::instructions::Instruction::*;
    let mut data = slice.iter();
    let Some(&opcode) = data.next() else {
        return "-".to_owned();
    };
    let op_lo = data.next();
    let op_hi = data.next();
    let ins = Instruction::from(opcode);
    match ins {
        Halt => "halt".into(),
        Illegal(bad_opcode) => format!("??? ({bad_opcode:#04X})"),
        Nop => "nop".into(),
        LoadRegImm8(reg) => format!("ld {reg}, {}", format_maybe_byte(op_lo)),
        LoadRegImm16(reg) => format!("ld {reg}, {}", format_maybe_word(op_lo, op_hi)),
    }
}

#[expect(clippy::single_call_fn, reason = "will be used more")]
fn format_maybe_byte(maybe_op: Option<&u8>) -> String {
    if let Some(op) = maybe_op {
        format!("{op:#04X}")
    } else {
        "??? (no operand)".to_owned()
    }
}

#[expect(clippy::single_call_fn, reason = "will be used more")]
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
    use anyhow::Context as _;

    use super::*;

    #[test]
    fn assembler_assembles_and_disassembles_instructions_correctly() {
        use Instruction::*;
        use Reg8::*;
        use Reg16::*;
        let cases: &[(&str, &[u8])] = &[
            ("nop", &[u8::from(Nop)]),
            ("halt", &[u8::from(Halt)]),
            ("ld a, 0xFF", &[u8::from(LoadRegImm8(A)), 0xFF]),
            ("ld b, 0xFF", &[u8::from(LoadRegImm8(B)), 0xFF]),
            ("ld c, 0xFF", &[u8::from(LoadRegImm8(C)), 0xFF]),
            ("ld d, 0xFF", &[u8::from(LoadRegImm8(D)), 0xFF]),
            ("ld e, 0xFF", &[u8::from(LoadRegImm8(E)), 0xFF]),
            ("ld f, 0xFF", &[u8::from(LoadRegImm8(F)), 0xFF]),
            ("ld g, 0xFF", &[u8::from(LoadRegImm8(G)), 0xFF]),
            ("ld h, 0xFF", &[u8::from(LoadRegImm8(H)), 0xFF]),
            ("ld ab, 0xBEEF", &[u8::from(LoadRegImm16(AB)), 0xEF, 0xBE]),
            ("ld cd, 0xBEEF", &[u8::from(LoadRegImm16(CD)), 0xEF, 0xBE]),
            ("ld ef, 0xBEEF", &[u8::from(LoadRegImm16(EF)), 0xEF, 0xBE]),
            ("ld gh, 0xBEEF", &[u8::from(LoadRegImm16(GH)), 0xEF, 0xBE]),
            // ("ld (0x00AF), a", &[LdMemByteA as u8, 0xAF, 0x00]),
            // ("ld (0x00AF), b", &[LdMemByteB as u8, 0xAF, 0x00]),
            // ("ld (0x00AF), c", &[LdMemByteC as u8, 0xAF, 0x00]),
            // ("ld (0x00AF), d", &[LdMemByteD as u8, 0xAF, 0x00]),
            // ("ld (0x00AF), e", &[LdMemByteE as u8, 0xAF, 0x00]),
            // ("ld (0x00AF), f", &[LdMemByteF as u8, 0xAF, 0x00]),
            // ("ld (0x00AF), g", &[LdMemByteG as u8, 0xAF, 0x00]),
            // ("ld (0x00AF), h", &[LdMemByteH as u8, 0xAF, 0x00]),
        ];
        for &(source, object) in cases {
            let generated = Assembler::from(source)
                .debug_assemble()
                .context(format!("assembling '{source}'"))
                .unwrap();
            assert_eq!(
                &generated,
                object,
                "wrong assembly for '{source}': want {}, got {}",
                as_hex(object),
                as_hex(&generated),
            );
            assert_eq!(
                &disassemble(object),
                source,
                "wrong disassembly for {}",
                as_hex(object)
            );
        }
    }

    #[test]
    fn disassembler_copes_with_invalid_code() {
        // Load immediate without a following operand
        assert_eq!(disassemble(&[0x10]), "ld a, ??? (no operand)");
        // Invalid opcode
        assert_eq!(disassemble(&[0x1C, 0xFF]), "??? (0x1C)");
    }

    fn as_hex(data: &[u8]) -> String {
        let mut byte_strs = Vec::new();
        for byte in data {
            byte_strs.push(format!("{byte:#04X}"));
        }
        format!("[{}]", byte_strs.join(", "))
    }
}
