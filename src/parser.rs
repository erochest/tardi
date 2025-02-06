use std::convert::TryFrom;

use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Integer(i64),
    String(String),
    Plus,
    Minus,
    Multiply,
    Division,
}

impl TryFrom<&str> for TokenType {
    type Error = Error;

    fn try_from(word: &str) -> Result<Self> {
        if let Ok(number) = word.parse::<i64>() {
            Ok(TokenType::Integer(number))
        } else if word == "+" {
            Ok(TokenType::Plus)
        } else if word == "-" {
            Ok(TokenType::Minus)
        } else if word == "*" {
            Ok(TokenType::Multiply)
        } else if word == "/" {
            Ok(TokenType::Division)
        } else if word.starts_with("\"") {
            Ok(TokenType::String(word.trim_matches('"').to_string()))
        } else {
            Err(Error::InvalidToken(word.to_string()))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line_no: usize,
    pub column: usize,
    pub length: usize,
}

pub fn parse(input: &str) -> Vec<Result<Token>> {
    input
        .split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let token_type = TokenType::try_from(word)?;

            Ok(Token {
                token_type,
                line_no: 1,
                column: i,
                length: word.len(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
