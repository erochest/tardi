use std::num::ParseIntError;
use std::str::{CharIndices, FromStr};

use tardi_core::value::{Value, ValueData};

#[derive(Debug)]
pub struct Scanner<'a> {
    last: Option<(usize, char)>,
    chars: CharIndices<'a>,
}

impl<'a> Scanner<'a> {
    pub fn from_string(input: &'a str) -> Self {
        let mut chars = input.char_indices();
        let last = chars.next();
        Self { last, chars }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.last {
                Some((_, c)) if !c.is_whitespace() => break,
                None => break,
                _ => {}
            }
            self.read_char();
        }
    }

    fn read_word(&mut self) -> Option<Value> {
        let start = self.last.map(|(i, _)| i).unwrap_or_default();
        let mut length = 0;
        let mut buffer = String::new();

        loop {
            match self.last {
                Some((_, c)) if c.is_whitespace() => break,
                Some((_, c)) => {
                    buffer.push(c);
                    length += c.len_utf8();
                }
                None => break,
            }
            self.read_char();
        }

        if buffer.is_empty() {
            return None;
        }
        // TODO: look ahead on '-' and '+'
        if let Some(c) = buffer.chars().next()
            && (c.is_ascii_digit() || c == '-' || c == '+')
            && let Some(number_data) = self.parse_full_number(&buffer)
        {
            return Some(Value::new(number_data, buffer, start, length));
        }
        Some(Value::new(
            ValueData::Word(buffer.clone()),
            buffer,
            start,
            length,
        ))
    }

    fn read_char(&mut self) -> &Option<(usize, char)> {
        self.last = self.chars.next();
        &self.last
    }

    fn push(&mut self, c: char, buffer: &mut String) -> usize {
        buffer.push(c);
        c.len_utf8()
    }

    fn push_read(&mut self, c: char, buffer: &mut String) -> usize {
        let length = self.push(c, buffer);
        self.read_char();
        length
    }

    fn read_string(&mut self) -> Option<Value> {
        let mut buffer = String::new();
        let mut value_data = String::new();
        let start = self.last.map(|(i, _)| i).unwrap_or_default();
        let mut length = self.last.map(|(_, c)| c.len_utf8()).unwrap_or_default();

        if let Some((_, c)) = self.last {
            buffer.push(c);
        }

        loop {
            self.read_char();
            match self.last {
                Some((_, c)) if c == '"' => {
                    length += self.push_read(c, &mut buffer);
                    break;
                }
                Some((_, c)) if c == '\\' => {
                    // \ " s
                    length += self.push_read(c, &mut buffer);
                    if let Some((_, c1)) = self.last {
                        value_data.push(c1);
                        length += self.push(c1, &mut buffer);
                    } else {
                        // TODO: error
                    }
                }
                Some((_, c)) => {
                    value_data.push(c);
                    length += self.push(c, &mut buffer);
                }
                None => {
                    // TODO:: error
                    break;
                }
            }
        }

        if buffer.is_empty() {
            None
        } else {
            Some(Value::new(
                ValueData::String(value_data),
                buffer,
                start,
                length,
            ))
        }
    }

    fn parse_full_number(&self, buffer: &str) -> Option<ValueData> {
        if let Some((numeric, type_code)) = buffer.split_once(':') {
            return match type_code {
                "i8" | "I8" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::I8(i as i8)),
                "i16" | "I16" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::I16(i as i16)),
                "i32" | "I32" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::I32(i as i32)),
                "i64" | "I64" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::I64(i as i64)),
                "i128" | "I128" => self.parse_signed_number(numeric).map(ValueData::I128),
                "isize" | "Isize" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::Isize(i as isize)),
                "u8" | "U8" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::U8(i as u8)),
                "u16" | "U16" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::U16(i as u16)),
                "u32" | "U32" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::U32(i as u32)),
                "u64" | "U64" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::U64(i as u64)),
                "u128" | "U128" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::U128(i as u128)),
                "usize" | "Usize" => self
                    .parse_signed_number(numeric)
                    .map(|i| ValueData::Usize(i as usize)),
                // TODO: this should probably be an error
                _ => self
                    .parse_signed_number(buffer)
                    .map(|i| ValueData::Isize(i as isize)),
            };
        }
        let parsed = self.parse_signed_number(buffer);
        parsed.map(|i| ValueData::Isize(i as isize))
    }

    fn parse_signed_number(&self, buffer: &str) -> Option<i128> {
        if buffer.starts_with("0x") || buffer.starts_with("0X") {
            i128::from_str_radix(&buffer[2..], 16).ok()
        } else if buffer.starts_with("0o") || buffer.starts_with("0O") {
            i128::from_str_radix(&buffer[2..], 8).ok()
        } else if buffer.starts_with("0b") || buffer.starts_with("0B") {
            i128::from_str_radix(&buffer[2..], 2).ok()
        } else {
            buffer
                .parse()
                .ok()
                .or_else(|| buffer.replace("_", "").replace(",", "").parse().ok())
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        match self.last {
            Some((_, '"')) => self.read_string(),
            _ => self.read_word(),
        }
    }
}

#[cfg(test)]
mod tests;
