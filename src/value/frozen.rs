use std::convert::TryFrom;
use std::fmt;
use std::hash::Hash;

use crate::error::{Error, VMError};
use crate::value::ValueData;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrozenValueData {
    Integer(i64),
    Boolean(bool),
    Char(char),
    String(String),
    Address(usize),
    Word(String),
    Symbol { module: String, word: String },
    Return(usize, bool),
}

impl FrozenValueData {
    pub fn to_repr(&self) -> String {
        if let FrozenValueData::String(ref s) = self {
            format!("\"{}\"", s.replace("\\", "\\\\").replace("\"", "\\\""))
        } else {
            self.to_string()
        }
    }
}

impl fmt::Display for FrozenValueData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrozenValueData::Integer(n) => write!(f, "{}", n),
            FrozenValueData::Boolean(true) => write!(f, "#t"),
            FrozenValueData::Boolean(false) => write!(f, "#f"),
            FrozenValueData::Char(c) => match c {
                '\n' => write!(f, "'\\n'"),
                '\r' => write!(f, "'\\r'"),
                '\t' => write!(f, "'\\t'"),
                '\\' => write!(f, "'\\\\'"),
                '\'' => write!(f, "'\\''"),
                c => write!(f, "'{}'", c),
            },
            FrozenValueData::String(s) => write!(f, "{}", s),
            FrozenValueData::Address(addr) => write!(f, "<@{}>", addr),
            FrozenValueData::Word(word) => write!(f, "{}", word),
            // TODO: escape word if it starts with punctuation (`module::\:` or something)
            FrozenValueData::Symbol { module, word } => write!(f, "{}::{}", module, word),
            FrozenValueData::Return(address, breakpoint) => {
                write!(f, "<@{} - {}>", address, breakpoint)
            }
        }
    }
}

impl TryFrom<ValueData> for FrozenValueData {
    type Error = Error;

    fn try_from(value: ValueData) -> Result<Self, Self::Error> {
        match value {
            ValueData::Integer(i) => Ok(FrozenValueData::Integer(i)),
            ValueData::Boolean(b) => Ok(FrozenValueData::Boolean(b)),
            ValueData::Char(c) => Ok(FrozenValueData::Char(c)),
            ValueData::String(s) => Ok(FrozenValueData::String(s)),
            ValueData::Address(a) => Ok(FrozenValueData::Address(a)),
            ValueData::Word(w) => Ok(FrozenValueData::Word(w)),
            ValueData::Symbol { module, word } => Ok(FrozenValueData::Symbol { module, word }),
            ValueData::Return(a, bp) => Ok(FrozenValueData::Return(a, bp)),
            ValueData::Float(_)
            | ValueData::List(_)
            | ValueData::HashMap(_)
            | ValueData::Function(_)
            | ValueData::Macro
            | ValueData::Literal(_)
            | ValueData::Writer(_)
            | ValueData::Reader(_)
            | ValueData::EndOfInput => Err(VMError::UnfreezableValue(value).into()),
        }
    }
}

impl From<FrozenValueData> for ValueData {
    fn from(value: FrozenValueData) -> Self {
        match value {
            FrozenValueData::Integer(i) => ValueData::Integer(i),
            FrozenValueData::Boolean(b) => ValueData::Boolean(b),
            FrozenValueData::Char(c) => ValueData::Char(c),
            FrozenValueData::String(s) => ValueData::String(s),
            FrozenValueData::Address(a) => ValueData::Address(a),
            FrozenValueData::Word(w) => ValueData::Word(w),
            FrozenValueData::Symbol { module, word } => ValueData::Symbol { module, word },
            FrozenValueData::Return(a, f) => ValueData::Return(a, f),
        }
    }
}

impl From<i64> for FrozenValueData {
    fn from(value: i64) -> Self {
        FrozenValueData::Integer(value)
    }
}

impl From<bool> for FrozenValueData {
    fn from(value: bool) -> Self {
        FrozenValueData::Boolean(value)
    }
}

impl From<char> for FrozenValueData {
    fn from(value: char) -> Self {
        FrozenValueData::Char(value)
    }
}

impl From<&str> for FrozenValueData {
    fn from(value: &str) -> Self {
        FrozenValueData::String(value.to_string())
    }
}

impl From<String> for FrozenValueData {
    fn from(value: String) -> Self {
        FrozenValueData::String(value)
    }
}
