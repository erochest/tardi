use std::str::Chars;

use tardi_core::value::Value;

#[derive(Debug)]
pub struct Scanner<'a> {
    last: Option<char>,
    chars: Chars<'a>,
}

impl<'a> Scanner<'a> {
    pub fn from_string(input: &'a str) -> Self {
        let mut chars = input.chars();
        let last = chars.next();
        Self { last, chars }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.last {
                Some(c) if !c.is_whitespace() => break,
                None => break,
                _ => {}
            }
            self.last = self.chars.next();
        }
    }

    fn read_word(&mut self) -> Option<Value> {
        let mut buffer = String::new();

        loop {
            match self.last {
                Some(c) if c.is_whitespace() => break,
                Some(c) => buffer.push(c),
                None => break,
            }
            self.last = self.chars.next();
        }

        if buffer.is_empty() {
            None
        } else {
            Some(Value::new(buffer))
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        self.read_word()
    }
}

#[cfg(test)]
mod tests;
