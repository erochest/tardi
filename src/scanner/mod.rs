pub mod error;

use crate::module::SANDBOX;
use crate::scanner::error::{ScannerError, ScannerResult};
use crate::value::{Value, ValueData};
use std::convert::TryFrom;
use std::iter::from_fn;
use std::path::{Path, PathBuf};
use std::{char, fs, result};

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
    Internal {
        name: String,
    },
}

impl Source {
    pub fn get_key(&self) -> String {
        match &self {
            Source::InputString => SANDBOX.to_string(),
            Source::ScriptFile { path } => path.file_stem().unwrap().to_string_lossy().to_string(),
            Source::Module { name, .. } => name.clone(),
            Source::Internal { name } => name.clone(),
        }
    }

    pub fn get_path(&self) -> Option<&Path> {
        match &self {
            Source::InputString => None,
            Source::ScriptFile { path } => Some(path),
            Source::Module { path, .. } => Some(path),
            Source::Internal { .. } => None,
        }
    }
}

impl TryFrom<Source> for Scanner {
    type Error = ScannerError;

    fn try_from(source: Source) -> result::Result<Self, ScannerError> {
        let mut scanner = Scanner::default();
        scanner.input = match source {
            Source::InputString => String::new(),
            Source::ScriptFile { path } => fs::read_to_string(&path)?,
            Source::Module { path, .. } => fs::read_to_string(&path)?,
            Source::Internal { .. } => String::new(),
        };
        scanner.chars = scanner.input.chars().collect();

        Ok(scanner)
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
    /// Creates a new Scanner from an input string.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to scan
    ///
    /// # Returns
    ///
    /// A new Scanner instance initialized with the input string
    pub fn from_input_string(input: &str) -> Self {
        let (input, chars) = input_chars(input);
        Scanner {
            source: Source::InputString,
            input,
            chars,
            ..Scanner::default()
        }
    }

    /// Creates a new Scanner from a script file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the script file
    /// * `input` - The content of the script file
    ///
    /// # Returns
    ///
    /// A new Scanner instance initialized with the script file content
    pub fn from_script(path: &Path, input: &str) -> Self {
        let path = path.to_path_buf();
        let (input, chars) = input_chars(input);
        Scanner {
            source: Source::ScriptFile { path },
            input,
            chars,
            ..Scanner::default()
        }
    }

    /// Creates a new Scanner from a module.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module
    /// * `path` - The path to the module file
    /// * `input` - The content of the module file
    ///
    /// # Returns
    ///
    /// A new Scanner instance initialized with the module content
    pub fn from_module(name: &str, path: &Path, input: &str) -> Self {
        let name = name.to_string();
        let path = path.to_path_buf();
        let (input, chars) = input_chars(input);
        Scanner {
            source: Source::Module { name, path },
            input,
            chars,
            ..Scanner::default()
        }
    }

    pub fn from_internal_module(name: &str, input: &str) -> Self {
        let name = name.to_string();
        let source = Source::Internal { name };
        let (input, chars) = input_chars(input);
        Scanner {
            source,
            input,
            chars,
            ..Scanner::default()
        }
    }

    pub fn set_input_string(&mut self, input: &str) {
        self.source = Source::InputString;
        self.input = input.to_string();
        self.chars = input.chars().collect();
    }

    /// Scans and returns the next value from the input.
    ///
    /// This method skips any whitespace before reading the next value. It handles
    /// various types of values including numbers, strings, characters, booleans,
    /// and words.
    ///
    /// # Returns
    ///
    /// * `Some(Ok(Value))` - If a value was successfully scanned
    /// * `Some(Err(Error))` - If there was an error while scanning
    /// * `None` - If the end of input was reached
    pub fn scan_value(&mut self) -> Option<ScannerResult<Value>> {
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
            let value = self.create_value(value_data, start_offset, start_line, start_column);
            log::trace!("Scanner::scan_value {:?}", value);
            value
        }))
    }

    /// Scans all values until the end of input is reached.
    ///
    /// # Returns
    ///
    /// A vector of ScannerResults, each containing either a Value or an Error
    pub fn scan_to_end(&mut self) -> Vec<ScannerResult<Value>> {
        from_fn(|| self.scan_value()).collect()
    }

    /// Scans a list of values until a specific value is encountered.
    ///
    /// This method collects values until it finds one that matches the provided
    /// value_data. The matching value is not included in the returned list.
    ///
    /// # Arguments
    ///
    /// * `value_data` - The ValueData to scan until
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// * `Ok(Vec<ScannerResult<Value>>)` - The list of values scanned
    /// * `Err(Error)` - If an error occurred or end of input was reached
    pub fn scan_value_list(
        &mut self,
        value_data: ValueData,
    ) -> ScannerResult<Vec<ScannerResult<Value>>> {
        let mut buffer = Vec::new();
        // This will often come in as a Symbol, but those aren't created
        // until after this point in the pipeline.
        let value_data = if let ValueData::Symbol { word, .. } = value_data {
            ValueData::Word(word.clone())
        } else {
            value_data
        };

        while let Some(value) = self.scan_value() {
            match value {
                Ok(value) if value.data == value_data => return Ok(buffer),
                _ => buffer.push(value),
            }
        }

        Err(ScannerError::UnexpectedEndOfInput)
    }

    /// Reads a string until a specific delimiter is encountered.
    ///
    /// This method reads characters into a buffer until it finds the specified
    /// delimiter sequence. The delimiter is consumed but not included in the
    /// returned string.
    ///
    /// # Arguments
    ///
    /// * `delimiter` - The string sequence to read until
    ///
    /// # Returns
    ///
    /// A Result containing either:
    /// * `Ok(String)` - The string read up to the delimiter
    /// * `Err(Error)` - If the delimiter was not found before end of input
    pub fn read_string_until(&mut self, delimiter: &str) -> ScannerResult<String> {
        let mut buffer = String::new();
        let delimiter_chars: Vec<char> = delimiter.chars().collect();
        let delimiter_len = delimiter_chars.len();

        while self.index < self.chars.len() {
            if self.chars[self.index..].starts_with(&delimiter_chars) {
                for _ in 0..delimiter_len {
                    self.next_char();
                }
                return Ok(buffer);
            }

            buffer.push(self.next_char().unwrap());
        }

        Err(ScannerError::UnexpectedEndOfInput)
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
    fn scan_hex_digits(&mut self, max_len: usize) -> ScannerResult<u32> {
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
            return Err(ScannerError::InvalidEscapeSequence(
                "Expected hexadecimal digits".to_string(),
            ));
        }

        Ok(value)
    }

    /// Processes an escape sequence in a string or character literal
    fn process_escape_sequence(&mut self) -> ScannerResult<char> {
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
                                None => Err(ScannerError::InvalidEscapeSequence(format!(
                                    "Invalid Unicode codepoint: {}",
                                    value
                                ))),
                            },
                            _ => Err(ScannerError::InvalidEscapeSequence(
                                "Expected closing '}'".to_string(),
                            )),
                        }
                    }
                    _ => {
                        // ASCII escape \uXX
                        let value = self.scan_hex_digits(2)?;
                        if value > 0x7F {
                            return Err(ScannerError::InvalidEscapeSequence(format!(
                                "ASCII value out of range: {}",
                                value
                            )));
                        }
                        Ok(char::from_u32(value).unwrap())
                    }
                }
            }
            Some(c) => Err(ScannerError::InvalidEscapeSequence(format!("\\{}", c))),
            None => Err(ScannerError::UnterminatedChar),
        }
    }

    /// Scans a character literal
    fn scan_char(&mut self) -> ScannerResult<ValueData> {
        let char = match self.next_char() {
            Some('\\') => self.process_escape_sequence().map(ValueData::Char),
            Some('\'') => Err(ScannerError::InvalidLiteral(
                "Empty character literal".to_string(),
            )),
            Some(c) => Ok(ValueData::Char(c)),
            None => Err(ScannerError::UnterminatedChar),
        };

        if self.next_char() != Some('\'') {
            return Err(ScannerError::UnterminatedChar);
        }

        char
    }

    /// Scans any type of string literal (regular or triple-quoted)
    fn scan_any_string(&mut self) -> ScannerResult<ValueData> {
        let mut string = String::new();
        let mut is_triple = false;
        let mut quote_count = 0;

        // Check for triple quotes
        if self.chars.get(self.index) == Some(&'"') && self.chars.get(self.index + 1) == Some(&'"')
        {
            is_triple = true;
            // Consume the remaining two quotes
            if self.next_char() != Some('"') || self.next_char() != Some('"') {
                return Err(ScannerError::InvalidLiteral(
                    "Expected triple quote".to_string(),
                ));
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

        Err(ScannerError::UnterminatedString)
    }

    /// Peeks at the next character without consuming it
    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    /// Consumes and returns the next character
    fn next_char(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.advance(c);
        Some(c)
    }

    /// Advances the scanner state after consuming a character
    fn advance(&mut self, c: char) {
        self.index += 1;
        self.offset += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
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
        // Try parsing in order of specificity
        // TODO: have it look in imported/defined names for one and use that module here?
        // TODO: parse `module/word` pairs into symbols
        self.parse_boolean(lexeme)
            .or_else(|| self.parse_number(lexeme))
            .unwrap_or_else(|| ValueData::Symbol {
                module: self.source.get_key(),
                word: lexeme.to_string(),
            })
    }

    fn parse_boolean(&self, lexeme: &str) -> Option<ValueData> {
        match lexeme {
            "#t" => Some(ValueData::Boolean(true)),
            "#f" => Some(ValueData::Boolean(false)),
            _ => None,
        }
    }

    fn parse_number(&self, lexeme: &str) -> Option<ValueData> {
        // First try integer parsing
        if let Ok(number) = lexeme.parse::<i64>() {
            return Some(ValueData::Integer(number));
        }

        // Then try float parsing
        if let Ok(number) = lexeme.parse::<f64>() {
            return Some(ValueData::Float(number));
        }

        // TODO: Future number format support:
        // - Binary (0b prefix)
        // - Octal (0o prefix)
        // - Hexadecimal (0x prefix)
        // - Rational type (e.g., 3/4)
        // - Exponential notation (e.g., 1e-10)
        // - Floats without leading digit (e.g., .5)
        None
    }

    /// Scans a word (any sequence of non-whitespace characters)
    fn scan_word(&mut self, first_char: char) -> ScannerResult<Option<ValueData>> {
        let word = self.scan_word_chars(first_char);

        // Handle special cases
        if word.starts_with("//") {
            self.skip_eol();
            return Ok(None);
        }

        match word.as_str() {
            "MACRO:" => Ok(Some(ValueData::Macro)),
            _ => Ok(Some(ValueData::Word(word))),
        }
    }

    /// Scans the characters that make up a word
    fn scan_word_chars(&mut self, first_char: char) -> String {
        let mut word = String::from(first_char);

        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                break;
            }
            word.push(c);
            self.next_char();
        }

        word
    }
}

fn input_chars(input: &str) -> (String, Vec<char>) {
    let input = input.to_string();
    let chars = input.chars().collect();
    (input, chars)
}

#[cfg(test)]
mod tests;
