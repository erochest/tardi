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
            None
        } else {
            Some(Value::new(buffer, start, length))
        }
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
                        length += self.push(c1, &mut buffer);
                    } else {
                        // TODO: error
                    }
                }
                Some((_, c)) => {
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
