use core::{
    iter::{self, Peekable},
    str::Chars,
};

use anyhow::{Result, bail};

use crate::instructions::{INSTRUCTIONS, Length, Opcode::*};

/// Keywords recognised by the assembler.
pub const KEYWORDS: &[&str] = &["halt", "ld", "nop"];

/// Assembles a given program.
#[non_exhaustive]
pub struct Assembler<'source> {
    /// Stores the source code being assembled.
    pub chars: Peekable<Chars<'source>>,
    /// Enables verbose debugging.
    pub debug: bool,
}

impl<'source> Assembler<'source> {
    /// Consumes the next source character.
    #[inline]
    pub fn advance(&mut self) {
        self.chars.next();
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "any unexpected token is illegal here"
    )]
    #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
    /// Assembles the source code.
    ///
    /// # Errors
    ///
    /// If the source is invalid.
    #[inline]
    pub fn assemble(&mut self) -> Result<Vec<u8>> {
        let mut code = Vec::new();
        while let Some(token) = self.next_token() {
            code.extend(match token {
                Token::Keyword(kw) if kw == "halt" => vec![Halt as u8],
                Token::Keyword(kw) if kw == "nop" => vec![Nop as u8],
                Token::Keyword(kw) if kw == "ld" => {
                    let Some(next) = self.next_token() else {
                        bail!("unexpected end of file")
                    };
                    let Token::Register(reg) = next else {
                        bail!("expected register, got {next:?}")
                    };
                    let opcode = match reg.as_str() {
                        "a" => LdAN,
                        "b" => LdBN,
                        "ab" => LdABN,
                        _ => bail!("invalid register {reg}"),
                    };
                    let Some(Token::Comma) = self.next_token() else {
                        bail!("expected comma")
                    };
                    if ["a", "b"].contains(&reg.as_str()) {
                        let Some(Token::ByteLiteral(operand)) = self.next_token() else {
                            bail!("expected byte literal")
                        };
                        vec![opcode as u8, operand]
                    } else {
                        let Some(Token::WordLiteral(operand)) = self.next_token() else {
                            bail!("expected word literal")
                        };
                        let [op_lo, op_hi] = operand.to_le_bytes();
                        vec![opcode as u8, op_lo, op_hi]
                    }
                }
                unexpected => bail!("unexpected token {unexpected:?}"),
            });
        }
        Ok(code)
    }

    /// Skips the given string (or as much of it as is present in the source).
    #[inline]
    pub fn chomp(&mut self, st: &str) -> Option<()> {
        for want in st.chars() {
            _ = self.chars.next_if(|&got| want == got)?;
        }
        Some(())
    }

    /// Prints a message if debug mode is on.
    #[inline]
    pub fn debug_print(&self, msg: impl AsRef<str>) {
        if self.debug {
            println!("asm: {}", msg.as_ref());
        }
    }

    /// Creates a new `Assembler` that will assemble `source`.
    #[must_use]
    #[inline]
    pub fn new(source: &'source str) -> Self {
        Self {
            chars: source.chars().peekable(),
            debug: false,
        }
    }

    /// Creates a new debug `Assembler` to assemble `source`.
    #[must_use]
    #[inline]
    pub fn new_with_debug(source: &'source str) -> Self {
        Self {
            chars: source.chars().peekable(),
            debug: true,
        }
    }

    /// Scans and returns the next token from the source code.
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

    /// Reads a comma token.
    #[inline]
    pub fn read_comma(&mut self) -> Option<Token> {
        self.advance();
        Some(Token::Comma)
    }

    /// Reads a hex literal token.
    #[inline]
    pub fn read_hex_literal(&mut self) -> Option<Token> {
        self.chomp("0x");
        let literal: String =
            iter::from_fn(|| self.chars.next_if(|ch| !ch.is_whitespace())).collect();
        Some(match u8::from_str_radix(&literal, 16) {
            Ok(val) => {
                self.debug_print(format!("byte literal '{literal}'"));
                Token::ByteLiteral(val)
            }
            Err(_) => match u16::from_str_radix(&literal, 16) {
                Ok(val) => {
                    self.debug_print(format!("word literal '{literal}'"));
                    Token::WordLiteral(val)
                }
                Err(_) => Token::Illegal(literal),
            },
        })
    }

    /// Reads an identifier, register name, or keyword.
    #[inline]
    pub fn read_identifier(&mut self) -> Option<Token> {
        let ident: String = iter::from_fn(|| self.chars.next_if(|ch| ch.is_alphabetic())).collect();
        Some(match ident.as_str() {
            "a" | "b" | "ab" => {
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

    /// Reads an illegal token.
    #[inline]
    pub fn read_illegal(&mut self, ch: char) -> Option<Token> {
        self.advance();
        Some(Token::Illegal(ch.to_string()))
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
    Comma,
    ByteLiteral(u8),
    Identifier(String),
    Illegal(String),
    Keyword(String),
    Register(String),
    WordLiteral(u16),
}

/// Disassembles a single instruction from `slice`.
#[inline]
pub fn disassemble(slice: &[u8]) -> String {
    let mut data = slice.iter();
    let Some(opcode) = data.next() else {
        return "???".to_owned();
    };
    let Some(ins) = INSTRUCTIONS.get(opcode) else {
        return "???".to_owned();
    };
    match ins.length {
        Length::OneByte => ins.name.to_owned(),
        Length::TwoBytes if let Some(operand) = data.next() => {
            format!("{}, {:#04X}", ins.name, operand)
        }
        Length::ThreeBytes if let (Some(op_lo), Some(op_hi)) = (data.next(), data.next()) => {
            format!(
                "{}, {:#06X}",
                ins.name,
                u16::from_le_bytes([*op_lo, *op_hi])
            )
        }
        _ => "???".to_owned(),
    }
}

#[expect(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
#[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
mod tests {
    use super::*;

    #[test]
    fn assemble_correctly_assembles_source() {
        let assemble = |source| Assembler::new_with_debug(source).assemble();
        assert_eq!(assemble("nop").unwrap(), &[Nop as u8]);
        assert_eq!(assemble("halt").unwrap(), &[Halt as u8]);
        assert_eq!(assemble("ld a, 0xFF").unwrap(), &[LdAN as u8, 0xFF]);
        assert_eq!(assemble("ld b, 0xFF").unwrap(), &[LdBN as u8, 0xFF]);
        assert_eq!(
            assemble("ld ab, 0xBEEF").unwrap(),
            &[LdABN as u8, 0xEF, 0xBE]
        );
    }

    #[test]
    fn disassemble_correctly_disassembles_instructions() {
        assert_eq!(disassemble(&[Nop as u8]), "nop");
        assert_eq!(disassemble(&[Halt as u8]), "halt");
        assert_eq!(disassemble(&[LdAN as u8, 0xFF]), "ld a, 0xFF");
        assert_eq!(disassemble(&[LdBN as u8, 0xFF]), "ld b, 0xFF");
        assert_eq!(disassemble(&[LdABN as u8, 0xEF, 0xBE]), "ld ab, 0xBEEF");
        assert_eq!(disassemble(&[0xFF]), "???");
    }
}
