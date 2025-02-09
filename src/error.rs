use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::result;

pub type Result<R> = result::Result<R, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidOpCode(u8),
    InvalidToken(String),
    InvalidOperands(String, String),
    StackUnderflow,
    DivideByZero,
    InvalidUnicodeChar,
}

use Error::*;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError(ref err) => err.fmt(f),
            InvalidOpCode(code) => write!(f, "Invalid opcode: {}", code),
            InvalidToken(token) => write!(f, "Invalid token: {}", token),
            InvalidOperands(a, b) => write!(f, "Cannot perform operation with {} and {}", a, b),
            StackUnderflow => write!(f, "Stack underflow"),
            DivideByZero => write!(f, "Divide by zero"),
            InvalidUnicodeChar => write!(f, "Invalid Unicode character"),
        }
    }
}

impl error::Error for Error {
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        IoError(err)
    }
}
