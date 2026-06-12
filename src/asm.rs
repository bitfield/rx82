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

    /// Assembles the code in `source`.
    ///
    /// # Errors
    ///
    /// If the source is invalid.
    #[inline]
    pub fn assemble(&mut self) -> Result<Vec<u8>> {
        let tokens = self.tokens();
        codegen(&tokens)
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
            println!("tokenize: {}", msg.as_ref());
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
        self.debug_print(format!("identifier '{ident}'"));
        Some(match ident.as_str() {
            "a" => Token::Register(ident),
            kw if KEYWORDS.contains(&kw) => Token::Keyword(ident),
            _ => Token::Identifier(ident),
        })
    }

    #[inline]
    pub fn read_illegal(&mut self, ch: char) -> Option<Token> {
        self.advance();
        Some(Token::Illegal(ch.to_string()))
    }

    #[inline]
    pub fn skip_whitespace(&mut self) -> Option<Token> {
        while self.chars.next_if(|ch| ch.is_whitespace()).is_some() {}
        None
    }

    #[inline]
    pub fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(next_char) = self.chars.peek() {
            let next = *next_char;
            if let Some(token) = match next {
                '0' => self.read_hex_literal(),
                ',' => self.read_comma(),
                ch if ch.is_alphabetic() => self.read_identifier(),
                ch if ch.is_whitespace() => self.skip_whitespace(),
                ch => self.read_illegal(ch),
            } {
                tokens.push(token);
            }
        }
        tokens
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

/// Returns the assembled bytes for the program `input`.
///
/// # Errors
///
/// If the program is invalid.
#[expect(clippy::pattern_type_mismatch, reason = "can't borrow here")]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "all unexpected tokens are invalid"
)]
#[inline]
pub fn codegen(input: &[Token]) -> Result<Vec<u8>> {
    let mut code = Vec::new();
    let mut tokens = input.iter();
    while let Some(token) = tokens.next() {
        code.extend(match token {
            Token::Keyword(kw) if kw == "halt" => vec![HALT],
            Token::Keyword(kw) if kw == "nop" => vec![NOP],
            Token::Keyword(kw) if kw == "ld" => {
                let Some(Token::Register(reg)) = tokens.next() else {
                    bail!("expected register")
                };
                let opcode = match reg.as_str() {
                    "a" => LDA_N,
                    _ => bail!("invalid register {reg}"),
                };
                let Some(Token::Comma) = tokens.next() else {
                    bail!("expected comma")
                };
                let Some(Token::HexLiteral(operand)) = tokens.next() else {
                    bail!("expected hex literal")
                };
                vec![opcode, *operand]
            }
            _ => bail!("unexpected {token:?}"),
        });
    }
    Ok(code)
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
        assert_eq!(assemble("halt").unwrap(), &[HALT]);
        assert_eq!(assemble("ld a, 0xFF").unwrap(), &[LDA_N, 0xFF]);
        assert_eq!(assemble("nop").unwrap(), &[NOP]);
    }

    #[test]
    fn codegen_produces_correct_machine_code() {
        assert_eq!(
            codegen(&[Token::Keyword("nop".to_owned())]).unwrap(),
            &[NOP]
        );
        assert_eq!(
            codegen(&[
                Token::Keyword("ld".to_owned()),
                Token::Register("a".to_owned()),
                Token::Comma,
                Token::HexLiteral(0xFF)
            ])
            .unwrap(),
            &[LDA_N, 0xFF]
        );
    }

    #[test]
    fn disassemble_correctly_disassembles_instructions() {
        assert_eq!(disassemble(&[HALT]), "halt");
        assert_eq!(disassemble(&[NOP]), "nop");
        assert_eq!(disassemble(&[0xFF]), "???");
        assert_eq!(disassemble(&[LDA_N, 0xFF]), "ld a, 0xFF");
    }

    #[test]
    fn tokenizer_correctly_tokenizes_source() {
        assert_eq!(tokenize("nop"), [Token::Keyword("nop".to_owned())]);
        assert_eq!(
            tokenize("ld a, 0xFF"),
            [
                Token::Keyword("ld".to_owned()),
                Token::Register("a".to_owned()),
                Token::Comma,
                Token::HexLiteral(0xFF)
            ]
        );
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

    #[inline]
    #[must_use]
    fn tokenize(source: &str) -> Vec<Token> {
        Assembler::new_with_debug(source).tokens()
    }
}
