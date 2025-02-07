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

pub fn parse(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < input.len() {
        let start = index;
        while index < input.len() && !input[index..].starts_with(char::is_whitespace) {
            index += 1;
        }

        let word = &input[start..index];
        let token_type = TokenType::try_from(word)?;

        tokens.push(Token {
            token_type,
            line_no: 1,
            column: start,
            length: word.len(),
        });

        while index < input.len() && input[index..].starts_with(char::is_whitespace) {
            index += 1;
        }
    }

    Ok(tokens)
}

