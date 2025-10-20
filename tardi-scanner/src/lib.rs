use std::str::CharIndices;

use tardi_core::value::Value;

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
            self.last = self.chars.next();
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
            self.last = self.chars.next();
        }

        if buffer.is_empty() {
            None
        } else {
            Some(Value::new(buffer, start, length))
        }
    }

    fn read_string(&mut self) -> Option<Value> {
        let mut buffer = String::new();
        let start = self.last.map(|(i, _)| i).unwrap_or_default();
        let mut length = self.last.map(|(_, c)| c.len_utf8()).unwrap_or_default();

        if let Some((_, c)) = self.last {
            buffer.push(c);
        }

        loop {
            self.last = self.chars.next();
            match self.last {
                Some((_, c)) if c == '"' => {
                    length += c.len_utf8();
                    buffer.push(c);
                    self.last = self.chars.next();
                    break;
                }
                Some((_, c)) => {
                    length += c.len_utf8();
                    buffer.push(c);
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
            Some(Value::new(buffer, start, length))
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
