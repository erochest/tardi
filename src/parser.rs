use std::convert::TryFrom;

use crate::error::{Error, Result};

const STRING_INITIALIZATION_CAPACITY: usize = 8;

// TODO: '+' prefix to numbers
// TODO: '_' in long integer numbers?

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Integer(i64),
    Float(f64),
    Rational(i64, u64),
    String(String),
    Plus,
    Minus,
    Multiply,
    Division,
}

impl TryFrom<&str> for TokenType {
    type Error = Error;

    fn try_from(word: &str) -> Result<Self> {
        let (number_word, multiplier) = if word.starts_with('-') {
            (&word[1..], -1)
        } else {
            (word, 1)
        };
        if number_word.starts_with("0x") || number_word.starts_with("0X") {
            let hex = number_word
                .trim_start_matches("0x")
                .trim_start_matches("0X");
            if let Ok(number) = i64::from_str_radix(hex, 16) {
                return Ok(TokenType::Integer(number * multiplier));
            }
        } else if number_word.starts_with("0o") || number_word.starts_with("0O") {
            let oct = number_word
                .trim_start_matches("0o")
                .trim_start_matches("0O");
            if let Ok(number) = i64::from_str_radix(oct, 8) {
                return Ok(TokenType::Integer(number * multiplier));
            }
        } else if number_word.starts_with("0b") || number_word.starts_with("0B") {
            let bin = number_word
                .trim_start_matches("0b")
                .trim_start_matches("0B");
            if let Ok(number) = i64::from_str_radix(bin, 2) {
                return Ok(TokenType::Integer(number * multiplier));
            }
        } else if word == "+" {
            return Ok(TokenType::Plus);
        } else if word == "-" {
            return Ok(TokenType::Minus);
        } else if word == "*" {
            return Ok(TokenType::Multiply);
        } else if word == "/" {
            return Ok(TokenType::Division);
        } else if number_word.starts_with(|c| char::is_digit(c, 10)) && number_word.contains('/') {
            if let Some((left, denominator)) = number_word.split_once('/') {
                if let Ok(denominator) = denominator.parse::<u64>() {
                    let (whole_number, numerator) = if let Some((whole, numer)) = left.split_once(&['+', '-'][..]) {
                        (whole.parse::<i64>().unwrap_or(0), numer.parse::<i64>().unwrap_or(0))
                    } else {
                        (0, left.parse::<i64>().unwrap_or(0))
                    };
                    let final_numerator = whole_number * denominator as i64 + numerator;
                    return Ok(TokenType::Rational(final_numerator * multiplier, denominator));
                }
            }
                    return Ok(TokenType::Rational(numerator * multiplier, denominator));
                }
            }
        };
        if let Ok(number) = number_word.parse::<i64>() {
            Ok(TokenType::Integer(number * multiplier))
        } else if let Ok(number) = number_word.parse::<f64>() {
            Ok(TokenType::Float(number * multiplier as f64))
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
            let (new_index, token) = if input[index..].starts_with(&['"', '"', '"']) {
                read_long_string(&input, index)?
            } else {
                read_string(&input, index)?
            };
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

fn read_string_until(input: &[char], index: usize, terminator: &[char]) -> Result<(usize, String)> {
    let start = index;
    let mut offset = terminator.len();
    let mut word = String::with_capacity(STRING_INITIALIZATION_CAPACITY);

    while start + offset < input.len() {
        if input[start + offset..].starts_with(terminator) {
            break;
        }

        let char_to_push = if input[start + offset] == '\\' && start + offset + 1 < input.len() {
            offset += 1;
            match input[start + offset] {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                'u' => {
                    let (unicode_offset, unicode_char) =
                        parse_unicode(&input[start + offset + 1..])?;
                    offset += unicode_offset;
                    unicode_char
                }
                c => c,
            }
        } else {
            input[start + offset]
        };
        word.push(char_to_push);
        offset += 1;
    }

    Ok((start + offset + terminator.len(), word))
}

fn read_long_string(input: &[char], index: usize) -> Result<(usize, Token)> {
    let (end, word) = read_string_until(input, index, &['"', '"', '"'])?;
    let token_type = TokenType::String(word);

    let token = Token {
        token_type,
        line_no: 1,
        column: index,
        length: end - index,
    };

    Ok((end, token))
}

fn read_string(input: &[char], index: usize) -> Result<(usize, Token)> {
    let (end, word) = read_string_until(input, index, &['"'])?;
    let token_type = TokenType::String(word);

    let token = Token {
        token_type,
        line_no: 1,
        column: index,
        length: end - index,
    };

    Ok((end, token))
}

fn parse_unicode(input: &[char]) -> Result<(usize, char)> {
    if input.is_empty() || input[0] != '{' {
        return Err(Error::InvalidUnicodeChar);
    }
    let mut hex_offset = 1; // Skip the opening '{'
    let mut hex_str = String::new();

    while hex_offset < input.len() {
        if input[hex_offset] == '}' {
            break;
        }
        if !input[hex_offset].is_ascii_hexdigit() {
            return Err(Error::InvalidUnicodeChar);
        }
        hex_str.push(input[hex_offset]);
        hex_offset += 1;
    }

    if hex_offset == input.len() {
        return Err(Error::InvalidUnicodeChar);
    }

    let hex_value = u32::from_str_radix(&hex_str, 16).map_err(|_| Error::InvalidUnicodeChar)?;
    let unicode_char = char::from_u32(hex_value).ok_or(Error::InvalidUnicodeChar)?;

    Ok((hex_offset + 1, unicode_char)) // Include the closing '}'
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
