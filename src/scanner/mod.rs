use super::Scan;

use crate::error::{Error, Result, ScannerError};
use crate::value::{Value, ValueData};
use std::{char, result};

/// Scanner that converts source text into a stream of tokens
pub struct Scanner {
    /// Source text being scanned
    source: String,

    /// Vector of source characters
    chars: Vec<char>,

    /// Index of current character.
    index: usize,

    /// Current line number (1-based)
    line: usize,

    /// Current column number (1-based)
    column: usize,

    // TODO: how is this different than `index`?
    /// Current offset from start of source (0-based)
    offset: usize,

    /// Has the scanner reached the EndOfInput for this source?
    end_of_input: bool,
}

impl Scanner {
    /// Creates a new Scanner for the given source text
    pub fn new() -> Self {
        let source = String::new();
        let chars = vec![];
        Scanner {
            source,
            chars,
            index: 0,
            line: 1,
            column: 1,
            offset: 0,
            end_of_input: false,
        }
    }

    /// Scans hexadecimal digits up to the specified length
    fn scan_hex_digits(&mut self, max_len: usize) -> Result<u32> {
        let mut value = 0u32;
        let mut count = 0;

        while let Some(&c) = self.chars.get(self.index) {
            if count >= max_len {
                break;
            }

            match c.to_digit(16) {
                Some(digit) => {
                    value = value * 16 + digit;
                    self.next_char();
                    count += 1;
                }
                None => break,
            }
        }

        if count == 0 {
            return Err(Error::ScannerError(ScannerError::InvalidEscapeSequence(
                "Expected hexadecimal digits".to_string(),
            )));
        }

        Ok(value)
    }

    /// Processes an escape sequence in a string or character literal
    fn process_escape_sequence(&mut self) -> Result<char> {
        match self.next_char() {
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('\\') => Ok('\\'),
            Some('\'') => Ok('\''),
            Some('"') => Ok('"'),
            Some('u') => {
                match self.peek() {
                    Some('{') => {
                        // Unicode escape \u{XXXX}
                        self.next_char(); // consume '{'
                        let value = self.scan_hex_digits(6)?;
                        match self.next_char() {
                            Some('}') => match char::from_u32(value) {
                                Some(c) => Ok(c),
                                None => {
                                    Err(Error::ScannerError(ScannerError::InvalidEscapeSequence(
                                        format!("Invalid Unicode codepoint: {}", value),
                                    )))
                                }
                            },
                            _ => Err(Error::ScannerError(ScannerError::InvalidEscapeSequence(
                                "Expected closing '}'".to_string(),
                            ))),
                        }
                    }
                    _ => {
                        // ASCII escape \uXX
                        let value = self.scan_hex_digits(2)?;
                        if value > 0x7F {
                            return Err(Error::ScannerError(ScannerError::InvalidEscapeSequence(
                                format!("ASCII value out of range: {}", value),
                            )));
                        }
                        Ok(char::from_u32(value).unwrap())
                    }
                }
            }
            Some(c) => Err(Error::ScannerError(ScannerError::InvalidEscapeSequence(
                format!("\\{}", c),
            ))),
            None => Err(Error::ScannerError(ScannerError::UnterminatedChar)),
        }
    }

    /// Scans a character literal
    fn scan_char(&mut self) -> Result<ValueData> {
        match self.next_char() {
            Some('\\') => self.process_escape_sequence().map(ValueData::Char),
            Some(c) => {
                if c == '\'' {
                    Err(Error::ScannerError(ScannerError::InvalidLiteral(
                        "Empty character literal".to_string(),
                    )))
                } else {
                    Ok(ValueData::Char(c))
                }
            }
            None => Err(Error::ScannerError(ScannerError::UnterminatedChar)),
        }
    }

    /// Scans a string literal
    fn scan_string(&mut self) -> Result<ValueData> {
        let mut string = String::new();

        while let Some(c) = self.next_char() {
            match c {
                '"' => return Ok(ValueData::String(string)),
                '\\' => {
                    let escaped_char = self.process_escape_sequence()?;
                    string.push(escaped_char);
                }
                _ => string.push(c),
            }
        }

        Err(Error::ScannerError(ScannerError::UnterminatedString))
    }

    /// Scans a triple-quoted string literal
    fn scan_long_string(&mut self) -> Result<ValueData> {
        // Consume the remaining two quotes
        if self.next_char() != Some('"') || self.next_char() != Some('"') {
            return Err(Error::ScannerError(ScannerError::InvalidLiteral(
                "Expected triple quote".to_string(),
            )));
        }

        let mut string = String::new();
        let mut quote_count = 0;

        while let Some(c) = self.next_char() {
            match c {
                '"' => {
                    quote_count += 1;
                    if quote_count == 3 {
                        return Ok(ValueData::String(string));
                    }
                }
                _ => {
                    // Add any accumulated quotes if this isn't part of the closing sequence
                    while quote_count > 0 {
                        string.push('"');
                        quote_count -= 1;
                    }
                    string.push(c);
                }
            }
        }

        Err(Error::ScannerError(ScannerError::UnterminatedString))
    }

    /// Advances the scanner state after consuming a character
    fn advance(&mut self, c: char) {
        self.offset += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    /// Peeks at the next character without consuming it
    fn peek(&mut self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    /// Consumes and returns the next character
    fn next_char(&mut self) -> Option<char> {
        let c = *self.chars.get(self.index)?;
        self.index += 1;
        self.advance(c);
        Some(c)
    }

    /// Skips whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            // Use next_char which handles both advancing and consuming
            self.next_char();
        }
    }

    /// Skips to the end of the line
    fn skip_eol(&mut self) {
        while let Some(c) = self.next_char() {
            if c == '\n' {
                break;
            }
        }
    }

    fn parse_word(&self, lexeme: &str) -> ValueData {
        if lexeme == "#t" {
            ValueData::Boolean(true)
        } else if lexeme == "#f" {
            ValueData::Boolean(false)
        } else if let Ok(number) = lexeme.parse::<i64>() {
            ValueData::Integer(number)
        } else if let Ok(number) = lexeme.parse::<f64>() {
            ValueData::Float(number)
        } else {
            ValueData::Word(lexeme.to_string())
        }
    }

    /// Scans a number (integer or float)
    fn scan_number(&mut self, first_digit: char) -> Result<ValueData> {
        let mut number = String::from(first_digit);
        let mut is_float = false;

        // TODO: Implement more number formats in the future:
        // - Binary (0b prefix)
        // - Octal (0o prefix)
        // - Hexadecimal (0x prefix)
        // - Rational type (e.g., 3/4)
        // - Exponential notation for floats (e.g., 1e-10)
        // - Floats without leading digit (e.g., .5)

        // Scan integer part
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                if c == '.' {
                    // Only allow one decimal point
                    if is_float {
                        return Err(Error::ScannerError(ScannerError::InvalidNumber(number)));
                    }
                    is_float = true;
                    number.push(c);
                    self.next_char();

                    // Must have at least one digit after decimal
                    if let Some(next) = self.peek() {
                        if !next.is_ascii_digit() {
                            return Err(Error::ScannerError(ScannerError::InvalidNumber(number)));
                        }
                    } else {
                        return Err(Error::ScannerError(ScannerError::InvalidNumber(number)));
                    }
                } else {
                    break;
                }
            } else {
                number.push(c);
                self.next_char();
            }
        }

        // Parse the number
        if is_float {
            match number.parse::<f64>() {
                Ok(n) => Ok(ValueData::Float(n)),
                Err(_) => Err(Error::ScannerError(ScannerError::InvalidNumber(number))),
            }
        } else {
            match number.parse::<i64>() {
                Ok(n) => Ok(ValueData::Integer(n)),
                Err(_) => Err(Error::ScannerError(ScannerError::InvalidNumber(number))),
            }
        }
    }

    /// Scans a potential boolean value
    fn scan_boolean(&mut self) -> Result<ValueData> {
        match self.next_char() {
            Some('t') => Ok(ValueData::Boolean(true)),
            Some('f') => Ok(ValueData::Boolean(false)),
            _ => Err(Error::ScannerError(ScannerError::InvalidLiteral(
                "#".to_string(),
            ))),
        }
    }

    /// Scans a word (any sequence of non-whitespace characters)
    fn scan_word(&mut self, first_char: char) -> Result<Option<ValueData>> {
        let mut word = String::from(first_char);

        // Scan the rest of the word
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                break;
            }
            word.push(c);
            self.next_char();
        }

        if word.starts_with("//") {
            self.skip_eol();
            return Ok(None);
        }

        if word == "MACRO:" {
            Ok(Some(ValueData::Macro))
        } else {
            Ok(Some(ValueData::Word(word)))
        }
    }

    pub fn scan_value_list(&mut self, delimiter: &ValueData) -> Result<Vec<Value>> {
        log::trace!("Scanner::scan_token_list {:?}", delimiter);
        let mut buffer = Vec::new();

        loop {
            if let Some(value) = self.next() {
                let value = value?;
                if value.data == *delimiter {
                    break;
                }
                buffer.push(value);
            } else {
                return Err(ScannerError::UnexpectedEndOfInput.into());
            }
        }

        log::trace!("Scanner::scan_token_list <<< {:?}", buffer);
        Ok(buffer)
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Scanner::new()
    }
}

impl Scan for Scanner {
    fn scan(&mut self, input: &str) -> Result<Vec<Result<Value>>> {
        self.set_source(input);
        let tokens = self.collect();
        Ok(tokens)
    }

    fn set_source(&mut self, input: &str) {
        self.source = input.to_string();
        self.chars = self.source.chars().collect();
        self.index = 0;
        self.line = 1;
        self.column = 1;
        self.offset = 0;
        self.end_of_input = false;
    }

    fn scan_value(&mut self) -> Option<Result<Value>> {
        let next = self.next();
        log::trace!("scanned {:?}", next);
        next
    }

    fn scan_values_until(&mut self, value_data: ValueData) -> Result<Vec<Result<Value>>> {
        let mut buffer = Vec::new();

        while let Some(value) = self.scan_value() {
            match value {
                Ok(value) if value.data == value_data => break,
                Ok(value) if value.data == ValueData::EndOfInput => {
                    return Err(ScannerError::UnexpectedEndOfInput.into())
                }
                _ => buffer.push(value),
            }
        }

        Ok(buffer)
    }

    fn read_string_until(&mut self, delimiter: &str) -> Result<String> {
        let delimiter: Vec<char> = delimiter.chars().collect();
        let mut buffer = Vec::new();

        loop {
            if let Some(c) = self.next_char() {
                buffer.push(c);
                if self.chars[self.index..].starts_with(&delimiter) {
                    for _ in delimiter {
                        let _ = self.next_char();
                    }
                    break;
                }
            } else {
                return Err(ScannerError::UnexpectedEndOfInput.into());
            }
        }

        Ok(buffer.iter().collect())
    }
}

impl Iterator for Scanner {
    type Item = result::Result<Value, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end_of_input {
            return None;
        }

        // Skip any whitespace before the next token
        self.skip_whitespace();

        // Record start position of token
        let start_line = self.line;
        let start_column = self.column;
        let start_offset = self.offset;

        // Get next character
        let c = self.next_char();

        if c.is_none() {
            self.end_of_input = true;
            return Some(Ok(Value::from_parts(
                ValueData::EndOfInput,
                "",
                self.line,
                self.column,
                self.offset,
                0,
            )));
        }

        // Create token based on character
        let c = c.unwrap();
        let result = if c == '"' {
            // Check for triple quotes by peeking at the next two characters
            let is_triple = self.chars.get(self.index) == Some(&'"')
                && self.chars.get(self.index + 1) == Some(&'"');

            if is_triple {
                self.scan_long_string()
            } else {
                self.scan_string()
            }
        } else if c == '\'' {
            let char_result = self.scan_char();
            if let Ok(ValueData::Char(_)) = char_result {
                // Consume the closing single quote
                if self.next_char() != Some('\'') {
                    return Some(Err(Error::ScannerError(ScannerError::UnterminatedChar)));
                }
            }
            char_result
        } else {
            match self.scan_word(c) {
                Ok(Some(ValueData::Word(w))) => Ok(self.parse_word(&w)),
                Ok(Some(vd)) => Ok(vd),
                Ok(None) => {
                    return self.next();
                }
                Err(err) => {
                    return Some(Err(err));
                }
            }
        };

        // Calculate token length
        let length = self.offset - start_offset;

        // Wrap successful tokens with position information
        Some(result.map(|value_data| {
            Value::from_parts(
                value_data,
                &self.source[start_offset..self.offset],
                start_line,
                start_column,
                start_offset,
                length,
            )
        }))
    }
}

// TODO: refactor into a new file
#[cfg(test)]
mod tests;
