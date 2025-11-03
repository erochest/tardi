use std::str::CharIndices;

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
            && let Some(number_data) = self.parse_number(&buffer)
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

    fn parse_number(&self, buffer: &str) -> Option<ValueData> {
        let parsed = if buffer.starts_with("0x") || buffer.starts_with("0X") {
            isize::from_str_radix(&buffer[2..], 16)
        } else if buffer.starts_with("0o") || buffer.starts_with("0O") {
            isize::from_str_radix(&buffer[2..], 8)
        } else if buffer.starts_with("0b") || buffer.starts_with("0B") {
            isize::from_str_radix(&buffer[2..], 2)
        } else {
            buffer
                .parse()
                .or_else(|_| buffer.replace("_", "").replace(",", "").parse())
        };
        parsed.ok().map(ValueData::Isize)
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
