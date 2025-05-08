use crate::error::Error;
use std::error;
use std::fmt;
use std::io;
use std::result;

/// Type alias for Result<T, Error> to make scanner-specific error handling clearer
pub type ScannerResult<T> = result::Result<T, ScannerError>;

#[derive(Debug)]
pub enum ScannerError {
    InvalidNumber(String),
    InvalidLiteral(String),
    UnexpectedCharacter(char),
    UnterminatedString,
    UnterminatedChar,
    InvalidEscapeSequence(String),
    IoError(io::Error),
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
            ScannerError::IoError(err) => err.fmt(f),
            ScannerError::UnexpectedEndOfInput => write!(f, "End of input"),
            ScannerError::NotInitialized => write!(f, "Scanner not initialized"),
        }
    }
}

impl error::Error for ScannerError {}

impl From<ScannerError> for Error {
    fn from(err: ScannerError) -> Error {
        if let ScannerError::IoError(err) = err {
            Error::IoError(err)
        } else {
            Error::ScannerError(err)
        }
    }
}

impl From<io::Error> for ScannerError {
    fn from(err: io::Error) -> Self {
        ScannerError::IoError(err)
    }
}
