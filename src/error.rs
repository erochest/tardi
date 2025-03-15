use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::result;

pub type Result<R> = result::Result<R, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    VMError(VMError),
    ScannerError(ScannerError),
}

#[derive(Debug)]
pub enum VMError {
    StackUnderflow,
    InvalidOpCode(usize),
    // Add more VM-specific errors as needed
}

use Error::*;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError(ref err) => err.fmt(f),
            VMError(ref err) => err.fmt(f),
            ScannerError(ref err) => err.fmt(f),
        }
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::InvalidOpCode(code) => write!(f, "Invalid opcode: {}", code),
        }
    }
}

impl error::Error for Error {}
impl error::Error for VMError {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        IoError(err)
    }
}

impl From<VMError> for Error {
    fn from(err: VMError) -> Error {
        VMError(err)
    }
}

#[derive(Debug)]
pub enum ScannerError {
    InvalidNumber(String),
    InvalidLiteral(String),
    UnexpectedCharacter(char),
    UnterminatedString,
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScannerError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            ScannerError::InvalidLiteral(s) => write!(f, "Invalid literal: {}", s),
            ScannerError::UnexpectedCharacter(c) => write!(f, "Unexpected character: {}", c),
            ScannerError::UnterminatedString => write!(f, "Unterminated string"),
        }
    }
}

impl error::Error for ScannerError {}

impl From<ScannerError> for Error {
    fn from(err: ScannerError) -> Error {
        ScannerError(err)
    }
}
