use crate::error::{Error, Result, ScannerError};
use crate::value::{Value, ValueData};
use std::char;
use std::iter::from_fn;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub enum Source {
    #[default]
    InputString,
    ScriptFile {
        path: PathBuf,
    },
    Module {
        name: String,
        path: PathBuf,
    },
}

impl Source {
    pub fn get_key(&self) -> String {
        match &self {
            Source::InputString => "<input>".to_string(),
            Source::ScriptFile { path } => path.to_string_lossy().to_string(),
            Source::Module { path, .. } => path.to_string_lossy().to_string(),
        }
    }
}

/// Scanner that converts source text into a stream of tokens
pub struct Scanner {
    /// The source that the input is from.
    pub source: Source,

    /// Source text being scanned
    input: String,

    /// Vector of source characters
    chars: Vec<char>,

    /// Index of current character (in `chars`).
    index: usize,

    /// Current line number (1-based)
    line: usize,

    /// Current column number (1-based)
    column: usize,

    /// Current offset from start of `source`
    offset: usize,
}

impl Default for Scanner {
    fn default() -> Self {
        let source = String::new();
        let chars = vec![];
        Scanner {
            source: Source::InputString,
            input: source,
            chars,
            index: 0,
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl Scanner {
    pub fn from_input_string(input: &str) -> Self {
        let input = input.to_string();
        let chars = input.chars().collect();
        Scanner {
            source: Source::InputString,
            input,
            chars,
            index: 0,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    pub fn from_script(path: &Path, input: &str) -> Self {
        let path = path.to_path_buf();
        let input = input.to_string();
        let chars = input.chars().collect();
        Scanner {
            source: Source::ScriptFile { path },
            input,
            chars,
            index: 0,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    pub fn from_module(name: &str, path: &Path, input: &str) -> Self {
        let name = name.to_string();
        let path = path.to_path_buf();
        let input = input.to_string();
        let chars = input.chars().collect();
        Scanner {
            source: Source::Module { name, path },
            input,
            chars,
            index: 0,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    pub fn scan_value(&mut self) -> Option<Result<Value>> {
        // Skip any whitespace before the next token
        self.skip_whitespace();

        // Check if we've reached the end of input
        if self.index >= self.chars.len() {
            return None;
        }

        // Record start position of token
        let start_line = self.line;
        let start_column = self.column;
        let start_offset = self.offset;

        let result = match self.next_char() {
            Some('"') => self.scan_any_string(),
            Some('\'') => self.scan_char(),
            Some(c) => match self.scan_word(c) {
                Ok(Some(ValueData::Word(w))) => Ok(self.parse_word(&w)),
                Ok(Some(vd)) => Ok(vd),
                Ok(None) => {
                    return self.scan_value();
                }
                Err(err) => Err(err),
            },
            None => {
                // This case should not occur due to the check at the beginning of the function
                unreachable!("Unexpected end of input")
            }
        };

        // Wrap successful tokens with position information
        Some(result.map(|value_data| {
            self.create_value(value_data, start_offset, start_line, start_column)
        }))
    }

    pub fn scan_to_end(&mut self) -> Vec<Result<Value>> {
        from_fn(|| self.scan_value()).collect()
    }

    pub fn scan_value_list(&mut self, value_data: ValueData) -> Result<Vec<Result<Value>>> {
        let mut buffer = Vec::new();

        while let Some(value) = self.scan_value() {
            match value {
                Ok(value) if value.data == value_data => break,
                _ => buffer.push(value),
            }
        }

        // If we reached the end without finding the delimiter, return an error
        if buffer.last().is_none() {
            return Err(ScannerError::UnexpectedEndOfInput.into());
        }

        Ok(buffer)
    }

    pub fn read_string_until(&mut self, delimiter: &str) -> Result<String> {
        let delimiter: Vec<char> = delimiter.chars().collect();
        let mut buffer = Vec::new();

        while self.index < self.chars.len() {
            let c = self.next_char().unwrap(); // Safe because we checked index < len
            buffer.push(c);
            if self.chars[self.index..].starts_with(&delimiter) {
                for _ in delimiter {
                    let _ = self.next_char();
                }
                return Ok(buffer.iter().collect());
            }
        }

        Err(ScannerError::UnexpectedEndOfInput.into())
    }

    fn create_value(
        &self,
        value_data: ValueData,
        start_offset: usize,
        start_line: usize,
        start_column: usize,
    ) -> Value {
        let length = self.offset - start_offset;
        Value::from_parts(
            value_data,
            &self.input[start_offset..self.offset],
            start_line,
            start_column,
            start_offset,
            length,
        )
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
        let char = match self.next_char() {
            Some('\\') => self.process_escape_sequence().map(ValueData::Char),
            Some('\'') => {
                Err(ScannerError::InvalidLiteral("Empty character literal".to_string()).into())
            }
            Some(c) => Ok(ValueData::Char(c)),
            None => Err(Error::ScannerError(ScannerError::UnterminatedChar)),
        };

        if self.next_char() != Some('\'') {
            return Err(ScannerError::UnterminatedChar.into());
        }

        char
    }

    /// Scans any type of string literal (regular or triple-quoted)
    fn scan_any_string(&mut self) -> Result<ValueData> {
        let mut string = String::new();
        let mut is_triple = false;
        let mut quote_count = 0;

        // Check for triple quotes
        if self.chars.get(self.index) == Some(&'"') && self.chars.get(self.index + 1) == Some(&'"') {
            is_triple = true;
            // Consume the remaining two quotes
            if self.next_char() != Some('"') || self.next_char() != Some('"') {
                return Err(Error::ScannerError(ScannerError::InvalidLiteral(
                    "Expected triple quote".to_string(),
                )));
            }
        }

        while let Some(c) = self.next_char() {
            match c {
                '"' => {
                    if is_triple {
                        quote_count += 1;
                        if quote_count == 3 {
                            return Ok(ValueData::String(string));
                        }
                    } else {
                        return Ok(ValueData::String(string));
                    }
                }
                '\\' if !is_triple => {
                    let escaped_char = self.process_escape_sequence()?;
                    string.push(escaped_char);
                }
                _ => {
                    if is_triple && quote_count > 0 {
                        // Add any accumulated quotes if this isn't part of the closing sequence
                        while quote_count > 0 {
                            string.push('"');
                            quote_count -= 1;
                        }
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
        // TODO: Implement more number formats in the future:
        // - Binary (0b prefix)
        // - Octal (0o prefix)
        // - Hexadecimal (0x prefix)
        // - Rational type (e.g., 3/4)
        // - Exponential notation for floats (e.g., 1e-10)
        // - Floats without leading digit (e.g., .5)
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

        // comment
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
}

#[cfg(test)]
mod tests;
