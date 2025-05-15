use std::convert::Infallible;
use std::error;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::result;

use crate::error::Error;
use crate::scanner::error::ScannerError;

pub type CompilerResult<R> = result::Result<R, CompilerError>;

#[derive(Debug)]
pub enum CompilerError {
    IOError(io::Error),
    UnsupportedToken(String),
    InvalidOperation(String),
    UnmatchedBrace,
    UndefinedWord(String),
    InvalidFunction(String),
    MissingEnvironment,
    ValueHasNoTokenType(String),
    ModuleNotFound(String),
    InvalidModulePath(PathBuf),
    ImportCycleError(String),
    InvalidState(String),
    ScannerError(ScannerError),
    TypeMismatch(String),
    Infallible,
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::IOError(err) => err.fmt(f),
            CompilerError::UnsupportedToken(s) => write!(f, "Unsupported token: {}", s),
            CompilerError::InvalidOperation(s) => write!(f, "Invalid operation: {}", s),
            CompilerError::UnmatchedBrace => write!(f, "Unmatched closing brace"),
            CompilerError::UndefinedWord(s) => write!(f, "Undefined word: {}", s),
            CompilerError::InvalidFunction(s) => write!(f, "Invalid function: {}", s),
            CompilerError::MissingEnvironment => write!(f, "Compiling with environment"),
            CompilerError::ValueHasNoTokenType(s) => write!(f, "Value has to TokenType: {}", s),
            CompilerError::ModuleNotFound(name) => write!(f, "Missing module '{}'", name),
            CompilerError::InvalidModulePath(path) => write!(f, "Invalid module path: {:?}", path),
            CompilerError::ImportCycleError(module_name) => {
                write!(f, "Import loop on {}", module_name)
            }
            CompilerError::InvalidState(s) => write!(f, "Invalid compiler state: {}", s),
            CompilerError::ScannerError(err) => err.fmt(f),
            CompilerError::TypeMismatch(s) => write!(f, "Type mismatch: {}", s),
            CompilerError::Infallible => write!(f, "this shouldn't happen"),
        }
    }
}

impl error::Error for CompilerError {}

impl From<CompilerError> for Error {
    fn from(err: CompilerError) -> Error {
        match err {
            CompilerError::IOError(io_error) => Error::IoError(io_error),
            CompilerError::ScannerError(scanner_error) => Error::ScannerError(scanner_error),
            CompilerError::Infallible => Error::InfallibleError,
            _ => Error::CompilerError(err),
        }
    }
}

impl From<Infallible> for CompilerError {
    fn from(_: Infallible) -> Self {
        CompilerError::Infallible
    }
}

impl From<ScannerError> for CompilerError {
    fn from(value: ScannerError) -> Self {
        CompilerError::ScannerError(value)
    }
}

impl From<io::Error> for CompilerError {
    fn from(err: io::Error) -> Self {
        CompilerError::IOError(err)
    }
}
