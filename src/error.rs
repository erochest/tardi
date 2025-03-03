use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::result;

use crate::scanner::TokenType;
use crate::value::Value;

pub type Result<R> = result::Result<R, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    FormatError(fmt::Error),
    InvalidOpCode(u8),
    InvalidToken(String),
    InvalidOperands(String, String),
    StackUnderflow,
    DivideByZero,
    InvalidUnicodeChar,
    TokenTypeNotValue(TokenType),
    InvalidValueType(Value),
    EndOfFile(TokenType),
    TooManyConstants,
    PrecedenceError,
    InvalidState(String),
    UndefinedWord(String),
    UncallableObject(Value),
}

use Error::*;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError(ref err) => err.fmt(f),
            FormatError(ref err) => err.fmt(f),
            InvalidOpCode(code) => write!(f, "Invalid opcode: {}", code),
            InvalidToken(token) => write!(f, "Invalid token: {}", token),
            InvalidOperands(a, b) => write!(f, "Cannot perform operation with {} and {}", a, b),
            StackUnderflow => write!(f, "Stack underflow"),
            DivideByZero => write!(f, "Divide by zero"),
            InvalidUnicodeChar => write!(f, "Invalid Unicode character"),
            TokenTypeNotValue(ref token_type) => {
                write!(f, "TokenType {:?} has no value", token_type)
            }
            InvalidValueType(value) => write!(f, "Invalid Value type for operation: {:?}", value),
            EndOfFile(token_type) => write!(f, "End of file. Expecting: {:?}", token_type),
            TooManyConstants => write!(f, "Too many constants defined. Max is {}", u8::MAX),
            PrecedenceError => write!(f, "Wrong precedence for expression"),
            InvalidState(message) => write!(f, "Invalid state: {}", message),
            UndefinedWord(name) => write!(f, "Unknown function: {}", name),
            UncallableObject(value) => write!(f, "Uncallable object: {:?}", value),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        IoError(err)
    }
}

impl From<fmt::Error> for Error {
    fn from(value: fmt::Error) -> Self {
        FormatError(value)
    }
}
