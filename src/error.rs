use std::convert::From;
use std::convert::Infallible;
use std::error;
use std::fmt;
use std::io;
use std::result;

use rustyline::error::ReadlineError;

use crate::scanner::error::ScannerError;

pub type Result<R> = result::Result<R, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    VMError(VMError),
    ScannerError(ScannerError),
    CompilerError(CompilerError),
    InvalidOpCode(usize),
    ReplError(ReadlineError),
    TomlError(toml::de::Error),
    ConfigReadError(figment::Error),
    InfallibleError,
}

#[derive(Debug)]
pub enum VMError {
    StackUnderflow,
    StackOverflow,
    ReturnStackUnderflow,
    ReturnStackOverflow,
    InvalidInstructionPointer(usize),
    InvalidOpCode(usize, usize),
    InvalidConstantIndex(usize),
    TypeMismatch(String),
    DivisionByZero,
    EmptyList,
    InvalidAddress(usize),
    InvalidWordCall(String),
    MissingModule,
    Exit,
}

use Error::*;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError(ref err) => err.fmt(f),
            VMError(ref err) => err.fmt(f),
            ScannerError(ref err) => err.fmt(f),
            CompilerError(ref err) => err.fmt(f),
            InvalidOpCode(code) => write!(f, "invalid op code: {}", code),
            ReplError(ref err) => err.fmt(f),
            TomlError(ref err) => err.fmt(f),
            ConfigReadError(ref err) => err.fmt(f),
            InfallibleError => unimplemented!("Error::InfallibleError"),
        }
    }
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::StackUnderflow => write!(f, "Stack underflow"),
            VMError::StackOverflow => write!(f, "Stack overflow"),
            VMError::InvalidInstructionPointer(ip) => write!(f, "Invalid IP: {}", ip),
            VMError::InvalidOpCode(ip, code) => write!(f, "Invalid opcode: {} @ {}", code, ip),
            VMError::InvalidConstantIndex(index) => write!(f, "Invalid constant index: {}", index),
            VMError::TypeMismatch(op) => write!(f, "Type mismatch in {} operation", op),
            VMError::DivisionByZero => write!(f, "Division by zero"),
            VMError::ReturnStackUnderflow => write!(f, "Return stack underflow"),
            VMError::ReturnStackOverflow => write!(f, "Return stack overflow"),
            VMError::EmptyList => write!(f, "Cannot split head of empty list"),
            VMError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
            VMError::InvalidWordCall(word) => write!(f, "Invalid word call: {}", word),
            VMError::MissingModule => write!(f, "No module"),
            VMError::Exit => todo!(),
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

impl From<ReadlineError> for Error {
    fn from(err: ReadlineError) -> Self {
        ReplError(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        TomlError(err)
    }
}

impl From<figment::Error> for Error {
    fn from(err: figment::Error) -> Self {
        ConfigReadError(err)
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        InfallibleError
    }
}

// TODO: move to compiler?
#[derive(Debug)]
pub enum CompilerError {
    UnsupportedToken(String),
    InvalidOperation(String),
    UnmatchedBrace,
    UndefinedWord(String),
    InvalidFunction(String),
    MissingEnvironment,
    ValueHasNoTokenType(String),
    ModuleNotFound(String),
    InvalidState(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::UnsupportedToken(s) => write!(f, "Unsupported token: {}", s),
            CompilerError::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
            CompilerError::UnmatchedBrace => write!(f, "Unmatched closing brace"),
            CompilerError::UndefinedWord(s) => write!(f, "Undefined word: {}", s),
            CompilerError::InvalidFunction(s) => write!(f, "Invalid function: {}", s),
            CompilerError::MissingEnvironment => write!(f, "Compiling with environment"),
            CompilerError::ValueHasNoTokenType(s) => write!(f, "Value has to TokenType: {}", s),
            CompilerError::ModuleNotFound(name) => write!(f, "Missing module '{}'", name),
            CompilerError::InvalidState(s) => write!(f, "Invalid compiler state: {}", s),
        }
    }
}

impl error::Error for CompilerError {}

impl From<CompilerError> for Error {
    fn from(err: CompilerError) -> Error {
        CompilerError(err)
    }
}
