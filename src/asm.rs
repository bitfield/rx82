use core::{
    iter::{self, Peekable},
    str::Chars,
};

use anyhow::{Result, bail};

use crate::instructions::{INSTRUCTIONS, LDA_N, NOP};

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Token {
    Comma,
    HexLiteral(u8),
    Illegal(String),
    Keyword(String),
    Register(String),
}

#[non_exhaustive]
pub struct Tokenizer<'source>(pub Peekable<Chars<'source>>);

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.0.next_if(|ch| ch.is_whitespace()).is_some() {}
        println!("examining '{}'", self.0.peek()?);
        let token = match *self.0.peek()? {
            '0' => self.read_hex_literal(),
            ',' => self.advance(Token::Comma),
            ch if ch.is_alphabetic() => self.read_alpha(),
            ch => self.advance(Token::Illegal(ch.to_string())),
        };
        Some(token)
    }
}

impl<'source> Tokenizer<'source> {
    #[inline]
    pub fn advance(&mut self, token: Token) -> Token {
        self.0.next();
        token
    }

    #[inline]
    pub fn chomp(&mut self, st: &str) -> Option<()> {
        for want in st.chars() {
            _ = self.0.next_if(|&got| want == got)?;
        }
        Some(())
    }

    #[inline]
    #[must_use]
    pub fn new(input: &'source str) -> Self {
        Self(input.chars().peekable())
    }

    #[inline]
    pub fn read_alpha(&mut self) -> Token {
        let word: String = iter::from_fn(|| self.0.next_if(|ch| ch.is_alphabetic())).collect();
        println!("read word '{word}'");
        match word.as_str() {
            "a" => Token::Register(word),
            _ => Token::Keyword(word),
        }
    }

    #[inline]
    pub fn read_hex_literal(&mut self) -> Token {
        self.chomp("0x");
        let word: String = iter::from_fn(|| self.0.next_if(|ch| !ch.is_whitespace())).collect();
        println!("read word '{word}'");
        match u8::from_str_radix(&word, 16) {
            Ok(val) => Token::HexLiteral(val),
            Err(_) => Token::Illegal(word),
        }
    }
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
pub fn codegen<'tokenstream, T>(input: T) -> Result<Vec<u8>>
where
    T: IntoIterator<Item = &'tokenstream Token>,
{
    let mut code = Vec::new();
    let mut tokens = input.into_iter();
    while let Some(token) = tokens.next() {
        code.extend(match token {
            Token::Keyword(kw) if kw == "nop" => vec![NOP],
            Token::Keyword(kw) if kw == "ld" => {
                let Some(Token::Register(reg)) = tokens.next() else {
                    bail!("syntax error")
                };
                let opcode = match reg.as_str() {
                    "a" => LDA_N,
                    _ => bail!("invalid register {reg}"),
                };
                let Some(Token::Comma) = tokens.next() else {
                    bail!("syntax error")
                };
                let Some(Token::HexLiteral(operand)) = tokens.next() else {
                    bail!("syntax error")
                };
                vec![opcode, *operand]
            }
            _ => bail!("invalid token {token:?}"),
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

#[must_use]
#[inline]
pub fn tokenize(source: &str) -> Vec<Token> {
    Tokenizer::new(source).collect()
}

#[expect(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
mod tests {
    use crate::instructions::{LDA_N, NOP};

    use super::*;

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
}
