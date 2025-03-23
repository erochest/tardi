mod token;
pub use token::{Token, TokenType};

use crate::error::{Error, ScannerError};
use std::iter::Peekable;
use std::str::Chars;

/// Scanner that converts source text into a stream of tokens
pub struct Scanner<'a> {
    /// Source text being scanned
    source: &'a str,

    /// Iterator over source characters
    chars: Peekable<Chars<'a>>,

    /// Current line number (1-based)
    line: usize,

    /// Current column number (1-based)
    column: usize,

    /// Current offset from start of source (0-based)
    offset: usize,
}

impl<'a> Scanner<'a> {
    /// Creates a new Scanner for the given source text
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Scans hexadecimal digits up to the specified length
    fn scan_hex_digits(&mut self, max_len: usize) -> Result<u32, Error> {
        let mut value = 0u32;
        let mut count = 0;

        while let Some(&c) = self.chars.peek() {
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
    fn process_escape_sequence(&mut self) -> Result<char, Error> {
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
    fn scan_char(&mut self) -> Result<TokenType, Error> {
        match self.next_char() {
            Some('\\') => self.process_escape_sequence().map(TokenType::Char),
            Some(c) => {
                if c == '\'' {
                    Err(Error::ScannerError(ScannerError::InvalidLiteral(
                        "Empty character literal".to_string(),
                    )))
                } else {
                    Ok(TokenType::Char(c))
                }
            }
            None => Err(Error::ScannerError(ScannerError::UnterminatedChar)),
        }
    }

    /// Scans a string literal
    fn scan_string(&mut self) -> Result<TokenType, Error> {
        let mut string = String::new();

        while let Some(c) = self.next_char() {
            match c {
                '"' => return Ok(TokenType::String(string)),
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
    fn scan_long_string(&mut self) -> Result<TokenType, Error> {
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
                        return Ok(TokenType::String(string));
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
        self.chars.peek().copied()
    }

    /// Consumes and returns the next character
    fn next_char(&mut self) -> Option<char> {
        let c = self.chars.next()?;
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

    /// Scans a number (integer or float)
    fn scan_number(&mut self, first_digit: char) -> Result<TokenType, Error> {
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
                Ok(n) => Ok(TokenType::Float(n)),
                Err(_) => Err(Error::ScannerError(ScannerError::InvalidNumber(number))),
            }
        } else {
            match number.parse::<i64>() {
                Ok(n) => Ok(TokenType::Integer(n)),
                Err(_) => Err(Error::ScannerError(ScannerError::InvalidNumber(number))),
            }
        }
    }

    /// Scans a potential boolean value
    fn scan_boolean(&mut self) -> Result<TokenType, Error> {
        match self.next_char() {
            Some('t') => Ok(TokenType::Boolean(true)),
            Some('f') => Ok(TokenType::Boolean(false)),
            _ => Err(Error::ScannerError(ScannerError::InvalidLiteral(
                "#".to_string(),
            ))),
        }
    }

    /// Scans a word (any sequence of non-whitespace characters)
    fn scan_word(&mut self, first_char: char) -> Result<Option<TokenType>, Error> {
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

        // Check for keywords and operators
        Ok(Some(match word.as_str() {
            // Stack operations
            "dup" => TokenType::Dup,
            "swap" => TokenType::Swap,
            "rot" => TokenType::Rot,
            "drop" => TokenType::Drop,

            // Return stack operations
            ">r" => TokenType::ToR,
            "r>" => TokenType::RFrom,
            "r@" => TokenType::RFetch,

            // Arithmetic operators
            "+" => TokenType::Plus,
            "-" => TokenType::Dash,
            "*" => TokenType::Star,
            "/" => TokenType::Slash,

            // Comparison operators
            "==" => TokenType::EqualEqual,
            "!=" => TokenType::BangEqual,
            "<" => TokenType::Less,
            ">" => TokenType::Greater,
            "<=" => TokenType::LessEqual,
            ">=" => TokenType::GreaterEqual,
            "!" => TokenType::Bang,

            // List operations
            "<list>" => TokenType::CreateList,
            "append" => TokenType::Append,
            "prepend" => TokenType::Prepend,
            "concat" => TokenType::Concat,
            "split-head!" => TokenType::SplitHead,

            // String operations
            "<string>" => TokenType::CreateString,
            ">string" => TokenType::ToString,
            "utf8>string" => TokenType::Utf8ToString,
            "string-concat" => TokenType::StringConcat,

            // If it's not a known operator or keyword, it's a word
            _ => TokenType::Word(word),
        }))
    }
}

impl Iterator for Scanner<'_> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip any whitespace before the next token
        self.skip_whitespace();

        // Record start position of token
        let start_line = self.line;
        let start_column = self.column;
        let start_offset = self.offset;

        // Get next character
        let c = self.next_char()?;

        // Create token based on character
        let result = match c {
            '0'..='9' => self.scan_number(c),
            '#' => self.scan_boolean(),
            '\'' => {
                let char_result = self.scan_char();
                if let Ok(TokenType::Char(_)) = char_result {
                    // Consume the closing single quote
                    if self.next_char() != Some('\'') {
                        return Some(Err(Error::ScannerError(ScannerError::UnterminatedChar)));
                    }
                }
                char_result
            }
            '"' => {
                // Check for triple quotes by peeking at the next two characters
                let is_triple = {
                    let mut chars = self.chars.clone();
                    chars.next() == Some('"') && chars.next() == Some('"')
                };

                if is_triple {
                    self.scan_long_string()
                } else {
                    self.scan_string()
                }
            }
            c => match self.scan_word(c) {
                Ok(Some(w)) => Ok(w),
                Ok(None) => {
                    return self.next();
                }
                Err(err) => {
                    return Some(Err(err));
                }
            },
        };

        // Calculate token length
        let length = self.offset - start_offset;

        // Wrap successful tokens with position information
        Some(result.map(|token_type| {
            Token::new(
                token_type,
                start_line,
                start_column,
                start_offset,
                length,
                self.source[start_offset..self.offset].to_string(),
            )
        }))
    }
}

// TODO: refactor into a new file
#[cfg(test)]
mod tests;
