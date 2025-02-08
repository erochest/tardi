use std::convert::TryFrom;

use crate::error::{Error, Result};

const STRING_INITIALIZATION_CAPACITY: usize = 8;

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
    let input: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < input.len() {
        let current = input[index];

        if current.is_whitespace() {
            index += skip_whitespace(&input[index..]);
        } else if current == '"' {
            let (new_index, token) = read_string(&input, index);
            index = new_index;
            tokens.push(token);
        } else {
            let (new_index, token) = read_word(&input, index);
            index = new_index;
            tokens.push(token);
        }
    }
    Ok(tokens)
}

fn read_word(input: &[char], index: usize) -> (usize, Token) {
    let start = index;
    let mut offset = 0;
    while start + offset < input.len() && !input[start + offset].is_whitespace() {
        offset += 1;
    }
    let end = start + offset;

    let word: String = input[start..end].iter().collect();
    let token_type = TokenType::try_from(&word[..]).unwrap();

    let token = Token {
        token_type,
        line_no: 1,
        column: start,
        length: offset,
    };

    (end, token)
}

fn read_string(input: &[char], index: usize) -> (usize, Token) {
    let start = index;
    let mut offset = 1;
    let mut word = String::with_capacity(STRING_INITIALIZATION_CAPACITY);
    while start + offset < input.len() && input[start + offset] != '"' {
        let current_char = input[start + offset];
        if current_char == '\\' && start + offset + 1 < input.len() {
            offset += 1;
        }
        word.push(input[start + offset]);
        offset += 1;
    }
    let end = start + offset + 1;
    let token_type = TokenType::String(word);

    let token = Token {
        token_type,
        line_no: 1,
        column: start,
        length: offset + 1,
    };

    (end, token)
}

fn skip_whitespace(input: &[char]) -> usize {
    let mut offset = 0;
    while offset < input.len() && input[offset].is_whitespace() {
        offset += 1;
    }
    offset
}

#[cfg(test)]
mod tests;
