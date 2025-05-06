use std::error;
use std::fmt;
use crate::error::Error;

/// Type alias for Result<T, Error> to make scanner-specific error handling clearer
pub type ScannerResult<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum ScannerError {
    InvalidNumber(String),
    InvalidLiteral(String),
    UnexpectedCharacter(char),
    UnterminatedString,
    UnterminatedChar,
    InvalidEscapeSequence(String),
    UnexpectedEndOfInput,
    NotInitialized,
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScannerError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            ScannerError::InvalidLiteral(s) => write!(f, "Invalid literal: {}", s),
            ScannerError::UnexpectedCharacter(c) => write!(f, "Unexpected character: {}", c),
            ScannerError::UnterminatedString => write!(f, "Unterminated string"),
            ScannerError::UnterminatedChar => write!(f, "Unterminated character literal"),
            ScannerError::InvalidEscapeSequence(s) => write!(f, "Invalid escape sequence: {}", s),
            ScannerError::UnexpectedEndOfInput => write!(f, "End of input"),
            ScannerError::NotInitialized => write!(f, "Scanner not initialized"),
        }
    }
}

impl error::Error for ScannerError {}

impl From<ScannerError> for Error {
    fn from(err: ScannerError) -> Error {
        Error::ScannerError(err)
    }
}
