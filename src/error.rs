use std::convert::From;
use std::convert::Infallible;
use std::error;
use std::fmt;
use std::io;
use std::result;

use rustyline::error::ReadlineError;

// use crate::compiler::error::CompilerError;
// use crate::scanner::error::ScannerError;
// use crate::value::ValueData;

pub type Result<R> = result::Result<R, Error>;
// pub type VMResult<R> = result::Result<R, VMError>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    // VMError(VMError),
    // // ScannerError(ScannerError),
    // // CompilerError(CompilerError),
    InvalidOpCode(usize),
    ReplError(ReadlineError),
    TomlError(toml::de::Error),
    MissingConfiguration,
    ConfigReadError(Box<figment::Error>),
    // TODO: uncomment after compiler
    // InfallibleError,
    // TODO: uncomment after value.io
    // TardiError(Box<dyn error::Error>),
}

// #[derive(Debug)]
// pub enum VMError {
// StackUnderflow,
// StackOverflow,
// ReturnStackUnderflow,
// ReturnStackOverflow,
// InvalidInstructionPointer(usize),
// InvalidOpCode(usize, usize),
// InvalidOpIndex(usize),
// InvalidConstantIndex(usize),
// TypeMismatch(String),
// DivisionByZero,
// EmptyList,
// InvalidAddress(usize),
// InvalidWordCall(String),
// MissingEnvironment,
// MissingModule,
// // UnfreezableValue(ValueData),
// Stop,
// Break,
// Continue,
// Bye,
// }

use Error::*;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError(err) => err.fmt(f),
            //     VMError(err) => err.fmt(f),
            //     // ScannerError(err) => err.fmt(f),
            //     // CompilerError(err) => err.fmt(f),
            InvalidOpCode(code) => write!(f, "invalid op code: {}", code),
            ReplError(err) => err.fmt(f),
            TomlError(err) => err.fmt(f),
            MissingConfiguration => write!(f, "missing configuration"),
            ConfigReadError(err) => err.fmt(f),
            //     InfallibleError => unimplemented!("Error::InfallibleError"),
            //     TardiError(err) => err.fmt(f),
        }
    }
}

// impl fmt::Display for VMError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// match self {
//     VMError::StackUnderflow => write!(f, "Stack underflow"),
//     VMError::StackOverflow => write!(f, "Stack overflow"),
//     VMError::InvalidInstructionPointer(ip) => write!(f, "Invalid IP: {}", ip),
//     VMError::InvalidOpCode(ip, code) => write!(f, "Invalid opcode: {} @ {}", code, ip),
//     VMError::InvalidOpIndex(index) => write!(f, "invalid op table index: {}", index),
//     VMError::InvalidConstantIndex(index) => write!(f, "Invalid constant index: {}", index),
//     VMError::TypeMismatch(op) => write!(f, "Type mismatch in {} operation", op),
//     VMError::DivisionByZero => write!(f, "Division by zero"),
//     VMError::ReturnStackUnderflow => write!(f, "Return stack underflow"),
//     VMError::ReturnStackOverflow => write!(f, "Return stack overflow"),
//     VMError::EmptyList => write!(f, "Cannot split head of empty list"),
//     VMError::InvalidAddress(addr) => write!(f, "Invalid address: {}", addr),
//     VMError::InvalidWordCall(word) => write!(f, "Invalid word call: {}", word),
//     VMError::MissingModule => write!(f, "No module"),
//     VMError::MissingEnvironment => write!(f, "No environment"),
//     // VMError::UnfreezableValue(v) => write!(f, "cannot freeze value {}", v),
//     VMError::Stop => unreachable!("VMError::Stop"),
//     VMError::Break => unreachable!("VMError::Break"),
//     VMError::Continue => unreachable!("VMError::Continue"),
//     VMError::Bye => write!(f, "bye now"),
// }
//         Ok(())
//     }
// }

impl error::Error for Error {}
// impl error::Error for VMError {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        IoError(err)
    }
}

// impl From<VMError> for Error {
//     fn from(err: VMError) -> Error {
//         VMError(err)
//     }
// }

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
        ConfigReadError(Box::new(err))
    }
}

// impl From<Infallible> for Error {
//     fn from(_: Infallible) -> Self {
//         InfallibleError
//     }
// }

#[cfg(test)]
mod tests {
    use std::io;

    use rustyline::error::ReadlineError;
    use serde::de::Error as SerdeErrorTrait;

    use crate::error::Error;

    fn test_wrapped_error<E>(wrapped: E, message_contains: &str)
    where
        Error: From<E>,
    {
        let error = Error::from(wrapped);
        test_error_message(error, message_contains);
    }

    fn test_error_message(error: Error, message_contains: &str) {
        let message = format!("{}", error);
        assert!(message.contains(message_contains));
    }

    #[test]
    fn error_wraps_ioerror() {
        let io_error = io::Error::new(io::ErrorKind::Other, "this is an error");
        test_wrapped_error(io_error, "this is an error");
    }

    #[test]
    fn error_captures_invalid_op_code() {
        let error = Error::InvalidOpCode(42);
        test_error_message(error, "invalid op code: 42");
    }

    #[test]
    fn error_wraps_readline_error() {
        let rl_error = ReadlineError::Eof;
        test_wrapped_error(rl_error, "");
    }

    #[test]
    fn error_wraps_toml_error() {
        let toml_error = toml::de::Error::missing_field("missing_field");
        test_wrapped_error(toml_error, "missing_field");
    }

    #[test]
    fn error_captures_missing_configuration() {
        let error = Error::MissingConfiguration;
        test_error_message(error, "missing configuration");
    }

    #[test]
    fn error_wraps_figment_config_error() {
        let figment_error = figment::Error::missing_field("missing_field");
        test_wrapped_error(figment_error, "missing_field");
    }
}
