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
            _ => Err(Error::ScannerError(ScannerError::InvalidLiteral("#".to_string()))),
        }
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
            _ => Err(Error::ScannerError(ScannerError::UnexpectedCharacter(c))),
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
mod tests {
    use super::*;

    #[test]
    fn test_scanner_position_tracking() {
        let mut scanner = Scanner::new("abc\ndef");

        // Initial position
        assert_eq!(scanner.line, 1);
        assert_eq!(scanner.column, 1);
        assert_eq!(scanner.offset, 0);

        // Advance through first line
        scanner.next_char(); // 'a'
        assert_eq!(scanner.line, 1);
        assert_eq!(scanner.column, 2);
        assert_eq!(scanner.offset, 1);

        scanner.next_char(); // 'b'
        assert_eq!(scanner.line, 1);
        assert_eq!(scanner.column, 3);
        assert_eq!(scanner.offset, 2);

        scanner.next_char(); // 'c'
        assert_eq!(scanner.line, 1);
        assert_eq!(scanner.column, 4);
        assert_eq!(scanner.offset, 3);

        scanner.next_char(); // '\n'
        assert_eq!(scanner.line, 2);
        assert_eq!(scanner.column, 1);
        assert_eq!(scanner.offset, 4);

        // First character of second line
        scanner.next_char(); // 'd'
        assert_eq!(scanner.line, 2);
        assert_eq!(scanner.column, 2);
        assert_eq!(scanner.offset, 5);
    }

    #[test]
    fn test_scanner_whitespace_handling() {
        let mut scanner = Scanner::new("   abc   \n   def");

        // Test initial whitespace skipping
        scanner.skip_whitespace();
        assert_eq!(scanner.offset, 3); // Skipped 3 spaces
        assert_eq!(scanner.column, 4); // Column should be 4 (1-based indexing)
        assert_eq!(scanner.line, 1); // Still on the first line

        // Test reading non-whitespace
        assert_eq!(scanner.next_char(), Some('a')); // 'a'
        assert_eq!(scanner.column, 5); // 'a' is at column 5
        assert_eq!(scanner.next_char(), Some('b')); // 'b'
        assert_eq!(scanner.next_char(), Some('c')); // 'c'

        // Test skipping spaces and newline
        scanner.skip_whitespace();
        assert_eq!(scanner.offset, 13); // Skipped 3 more spaces
        assert_eq!(scanner.column, 4); // Column 4 on the second line
        assert_eq!(scanner.line, 2); // Still on the second line

        // Test skipping when there is no whitespace
        scanner.skip_whitespace();
        assert_eq!(scanner.offset, 13); // Skipped 3 more spaces
        assert_eq!(scanner.column, 4); // Column 4 on the second line
        assert_eq!(scanner.line, 2); // Still on the second line

        // Test reading after whitespace
        assert_eq!(scanner.next_char(), Some('d')); // 'd'
        assert_eq!(scanner.column, 5); // 'd' is at column 5 of the second line
    }

    #[test]
    fn test_scan_integers() {
        let mut scanner = Scanner::new("42 123 0 -1");

        // Test "42"
        // TODO: refactor away from this pattern that uses an explicit `panic!`
        if let Some(Ok(token)) = scanner.next() {
            assert!(matches!(token.token_type, TokenType::Integer(42)));
            assert_eq!(token.line, 1);
            assert_eq!(token.column, 1);
            assert_eq!(token.length, 2);
            assert_eq!(token.lexeme, "42");
        } else {
            panic!("Failed to scan integer");
        }

        // Test "123"
        if let Some(Ok(token)) = scanner.next() {
            assert!(matches!(token.token_type, TokenType::Integer(123)));
            assert_eq!(token.lexeme, "123");
        } else {
            panic!("Failed to scan integer");
        }
    }

    #[test]
    fn test_scan_floats() {
        let mut scanner = Scanner::new("3.14 2.0 0.123");

        // Test "3.14"
        if let Some(Ok(token)) = scanner.next() {
            assert!(matches!(token.token_type, TokenType::Float(3.14)));
            assert_eq!(token.line, 1);
            assert_eq!(token.column, 1);
            assert_eq!(token.length, 4);
            assert_eq!(token.lexeme, "3.14");
        } else {
            panic!("Failed to scan float");
        }

        // Test invalid float formats
        let mut scanner = Scanner::new("3. .14");
        assert!(scanner.next().unwrap().is_err()); // "3." is invalid
        assert!(scanner.next().unwrap().is_err()); // ".14" is invalid (no leading digit)
    }

    #[test]
    fn test_scan_booleans() {
        let mut scanner = Scanner::new("#t #f #x");

        // Test "#t"
        if let Some(Ok(token)) = scanner.next() {
            assert!(matches!(token.token_type, TokenType::Boolean(true)));
            assert_eq!(token.line, 1);
            assert_eq!(token.column, 1);
            assert_eq!(token.length, 2);
            assert_eq!(token.lexeme, "#t");
        } else {
            panic!("Failed to scan boolean");
        }

        // Test "#f"
        if let Some(Ok(token)) = scanner.next() {
            assert!(matches!(token.token_type, TokenType::Boolean(false)));
            assert_eq!(token.lexeme, "#f");
        } else {
            panic!("Failed to scan boolean");
        }

        // Test invalid boolean
        assert!(scanner.next().unwrap().is_err()); // "#x" is not a valid boolean
    }
}
