use core::{
    iter::{self, Peekable},
    str::Chars,
};

use anyhow::{Result, bail};

use crate::instructions::{HALT, INSTRUCTIONS, LDA_N, NOP};

pub const KEYWORDS: &[&str] = &["halt", "ld", "nop"];

#[non_exhaustive]
pub struct Assembler<'source> {
    pub chars: Peekable<Chars<'source>>,
    pub debug: bool,
}

impl<'source> Assembler<'source> {
    #[inline]
    pub fn advance(&mut self) {
        self.chars.next();
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "any unexpected token is illegal here"
    )]
    /// Assembles the code in `self.source`.
    ///
    /// # Errors
    ///
    /// If the source is invalid.
    #[inline]
    pub fn assemble(&mut self) -> Result<Vec<u8>> {
        let mut code = Vec::new();
        while let Some(token) = self.next_token() {
            code.extend(match token {
                Token::Keyword(kw) if kw == "halt" => vec![HALT],
                Token::Keyword(kw) if kw == "nop" => vec![NOP],
                Token::Keyword(kw) if kw == "ld" => {
                    let Some(next) = self.next_token() else {
                        bail!("unexpected end of file")
                    };
                    let Token::Register(reg) = next else {
                        bail!("expected register, got {next:?}")
                    };
                    let opcode = match reg.as_str() {
                        "a" => LDA_N,
                        _ => bail!("invalid register {reg}"),
                    };
                    let Some(Token::Comma) = self.next_token() else {
                        bail!("expected comma")
                    };
                    let Some(Token::HexLiteral(operand)) = self.next_token() else {
                        bail!("expected hex literal")
                    };
                    vec![opcode, operand]
                }
                unexpected => bail!("unexpected token {unexpected:?}"),
            });
        }
        Ok(code)
    }

    #[inline]
    pub fn chomp(&mut self, st: &str) -> Option<()> {
        for want in st.chars() {
            _ = self.chars.next_if(|&got| want == got)?;
        }
        Some(())
    }

    #[inline]
    pub fn debug_print(&self, msg: impl AsRef<str>) {
        if self.debug {
            println!("asm: {}", msg.as_ref());
        }
    }

    #[must_use]
    #[inline]
    pub fn new(source: &'source str) -> Self {
        Self {
            chars: source.chars().peekable(),
            debug: false,
        }
    }

    #[must_use]
    #[inline]
    pub fn new_with_debug(source: &'source str) -> Self {
        Self {
            chars: source.chars().peekable(),
            debug: true,
        }
    }

    #[inline]
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if let Some(next_char) = self.chars.peek() {
            let next = *next_char;
            match next {
                '0' => self.read_hex_literal(),
                ',' => self.read_comma(),
                ch if ch.is_alphabetic() => self.read_identifier(),
                ch => self.read_illegal(ch),
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn read_comma(&mut self) -> Option<Token> {
        self.advance();
        Some(Token::Comma)
    }

    #[inline]
    pub fn read_hex_literal(&mut self) -> Option<Token> {
        self.chomp("0x");
        let literal: String =
            iter::from_fn(|| self.chars.next_if(|ch| !ch.is_whitespace())).collect();
        self.debug_print(format!("hex literal '{literal}'"));
        Some(match u8::from_str_radix(&literal, 16) {
            Ok(val) => Token::HexLiteral(val),
            Err(_) => Token::Illegal(literal),
        })
    }

    #[inline]
    pub fn read_identifier(&mut self) -> Option<Token> {
        let ident: String = iter::from_fn(|| self.chars.next_if(|ch| ch.is_alphabetic())).collect();
        Some(match ident.as_str() {
            "a" => {
                self.debug_print(format!("register '{ident}'"));
                Token::Register(ident)
            }
            kw if KEYWORDS.contains(&kw) => {
                self.debug_print(format!("keyword '{ident}'"));
                Token::Keyword(ident)
            }
            _ => {
                self.debug_print(format!("identifier '{ident}'"));
                Token::Identifier(ident)
            }
        })
    }

    #[inline]
    pub fn read_illegal(&mut self, ch: char) -> Option<Token> {
        self.advance();
        Some(Token::Illegal(ch.to_string()))
    }

    #[inline]
    pub fn skip_whitespace(&mut self) {
        while self.chars.next_if(|ch| ch.is_whitespace()).is_some() {}
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Token {
    Comma,
    HexLiteral(u8),
    Identifier(String),
    Illegal(String),
    Keyword(String),
    Register(String),
}

#[inline]
pub fn disassemble(slice: &[u8]) -> String {
    let mut data = slice.iter();
    let Some(opcode) = data.next() else {
        return "???".to_owned();
    };
    let Some(ins) = INSTRUCTIONS.get(opcode) else {
        return "???".to_owned();
    };
    match ins.bytes {
        1 => ins.name.to_owned(),
        2 if let Some(operand) = data.next() => format!("{}, {:#04X}", ins.name, operand),
        _ => "???".to_owned(),
    }
}

#[expect(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
mod tests {
    use crate::instructions::{HALT, LDA_N, NOP};

    use super::*;

    #[test]
    fn assemble_correctly_assembles_source() {
        assert_eq!(assemble("ld a, 0xFF").unwrap(), &[LDA_N, 0xFF]);
        assert_eq!(assemble("halt").unwrap(), &[HALT]);
        assert_eq!(assemble("nop").unwrap(), &[NOP]);
    }

    #[test]
    fn disassemble_correctly_disassembles_instructions() {
        assert_eq!(disassemble(&[HALT]), "halt");
        assert_eq!(disassemble(&[NOP]), "nop");
        assert_eq!(disassemble(&[0xFF]), "???");
        assert_eq!(disassemble(&[LDA_N, 0xFF]), "ld a, 0xFF");
    }

    /// Assembles the code in `source`.
    ///
    /// # Errors
    ///
    /// If the source is invalid.
    #[inline]
    fn assemble(source: &str) -> Result<Vec<u8>> {
        Assembler::new_with_debug(source).assemble()
    }
}
