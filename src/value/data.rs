use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};

use crate::error::{Result, VMError};
use crate::value::frozen::FrozenValueData;
use crate::value::lambda::Lambda;

use super::{SharedValue, TardiReader, TardiWriter, Value};

// -- they're more closely tied to `Environment` and they're part of what
// bridges across layers
// TODO: Have a Value member for doc comments so we can grab those in macros
// TODO: cache common values like small numbers, booleans, and empty collections.
// TODO: immutable tuple type
/// Enum representing different types of values that can be stored on the stack
#[derive(Debug, Clone)]
pub enum ValueData {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),
    String(String),
    // TODO: rename to Vector
    List(Vec<SharedValue>),
    HashMap(HashMap<FrozenValueData, SharedValue>),
    Function(Lambda),
    Address(usize),
    Word(String),
    Symbol { module: String, word: String },
    Macro,
    Literal(Box<Value>),
    Return(usize, bool),
    Writer(TardiWriter),
    Reader(TardiReader),
    EndOfInput,
}

impl ValueData {
    pub fn to_repr(&self) -> String {
        if let ValueData::String(ref s) = self {
            format!("\"{}\"", s.replace("\\", "\\\\").replace("\"", "\\\""))
        } else {
            self.to_string()
        }
    }

    pub fn get_word(&self) -> Option<&str> {
        if let ValueData::Word(ref w) = self {
            Some(w)
        } else if let ValueData::Symbol { word: ref w, .. } = self {
            Some(w)
        } else {
            None
        }
    }

    pub fn as_writer(&self) -> Option<&TardiWriter> {
        if let Self::Writer(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_writer_mut(&mut self) -> Option<&mut TardiWriter> {
        if let Self::Writer(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_reader(&self) -> Option<&TardiReader> {
        if let Self::Reader(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_reader_mut(&mut self) -> Option<&mut TardiReader> {
        if let Self::Reader(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value data is [`HashMap`].
    ///
    /// [`HashMap`]: ValueData::HashMap
    pub fn is_hash_map(&self) -> bool {
        matches!(self, Self::HashMap(..))
    }

    pub fn as_hash_map(&self) -> Option<&HashMap<FrozenValueData, SharedValue>> {
        if let Self::HashMap(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_hash_map_mut(&mut self) -> Option<&mut HashMap<FrozenValueData, SharedValue>> {
        if let Self::HashMap(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

// From implementations for each of the contained values
impl From<i64> for ValueData {
    fn from(value: i64) -> Self {
        ValueData::Integer(value)
    }
}

impl From<f64> for ValueData {
    fn from(value: f64) -> Self {
        ValueData::Float(value)
    }
}

impl From<bool> for ValueData {
    fn from(value: bool) -> Self {
        ValueData::Boolean(value)
    }
}

impl From<char> for ValueData {
    fn from(value: char) -> Self {
        ValueData::Char(value)
    }
}

impl From<&str> for ValueData {
    fn from(value: &str) -> Self {
        ValueData::String(value.to_string())
    }
}

impl From<String> for ValueData {
    fn from(value: String) -> Self {
        ValueData::String(value)
    }
}

impl From<ValueData> for Value {
    fn from(value: ValueData) -> Self {
        Value::new(value)
    }
}

impl fmt::Display for ValueData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueData::Integer(n) => write!(f, "{}", n),
            ValueData::Float(x) => {
                let s = format!("{}", x);
                if !s.contains('.') {
                    write!(f, "{}.0", s)
                } else {
                    write!(f, "{}", s)
                }
            }
            ValueData::Boolean(true) => write!(f, "#t"),
            ValueData::Boolean(false) => write!(f, "#f"),
            ValueData::Char(c) => match c {
                '\n' => write!(f, "'\\n'"),
                '\r' => write!(f, "'\\r'"),
                '\t' => write!(f, "'\\t'"),
                '\\' => write!(f, "'\\\\'"),
                '\'' => write!(f, "'\\''"),
                c => write!(f, "'{}'", c),
            },
            ValueData::List(list) => {
                write!(f, "{{")?;
                for item in list.iter() {
                    write!(f, " {}", item.borrow().to_repr())?;
                }
                write!(f, " }}")
            }
            ValueData::HashMap(hash_map) => {
                write!(f, "H{{")?;
                let mut pairs = hash_map.iter().collect::<Vec<_>>();
                pairs.sort();
                for (k, v) in pairs.iter() {
                    write!(f, " {{ {} {} }}", k.to_repr(), v.borrow().to_repr())?;
                }
                write!(f, " }}")
            }
            ValueData::String(s) => write!(f, "{}", s),
            ValueData::Function(lambda) => write!(f, "{}", lambda),
            ValueData::Address(addr) => write!(f, "<@{}>", addr),
            ValueData::Word(word) => write!(f, "{}", word),
            // TODO: escape word if it starts with punctuation (`module::\:` or something)
            ValueData::Symbol { module, word } => write!(f, "{}::{}", module, word),
            ValueData::Macro => write!(f, "MACRO:"),
            ValueData::Literal(value) => write!(f, "\\ {}", value),
            ValueData::Return(address, breakpoint) => write!(f, "<@{} - {}>", address, breakpoint),
            ValueData::Writer(writer) => write!(f, "{}", writer),
            ValueData::Reader(reader) => write!(f, "{}", reader),
            ValueData::EndOfInput => write!(f, "<EOI>"),
        }
    }
}

impl Eq for ValueData {}

impl Ord for ValueData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

// TODO: should symbols also match on strings?
impl PartialEq for ValueData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueData::Integer(a), ValueData::Integer(b)) => a == b,
            (ValueData::Float(a), ValueData::Float(b)) => a == b,
            (ValueData::Integer(a), ValueData::Float(b)) => (*a as f64) == *b,
            (ValueData::Float(a), ValueData::Integer(b)) => *a == (*b as f64),
            (ValueData::Boolean(a), ValueData::Boolean(b)) => a == b,
            (ValueData::Char(a), ValueData::Char(b)) => a == b,
            (ValueData::String(a), ValueData::String(b)) => a == b,
            (ValueData::List(a), ValueData::List(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(x, y)| *x.borrow() == *y.borrow())
            }
            (ValueData::HashMap(hm1), ValueData::HashMap(hm2)) => {
                hm1.len() == hm2.len()
                    && hm1
                        .iter()
                        .all(|(k, v)| hm2.get(k).is_some_and(|v2| v.borrow().eq(&v2.borrow())))
            }
            (ValueData::Function(a), ValueData::Function(b)) => a == b,
            (ValueData::Address(a), ValueData::Address(b)) => a == b,
            (ValueData::Word(a), ValueData::Word(b)) => a == b,
            (ValueData::Word(a), ValueData::Symbol { word: b, .. }) => a == b,
            (ValueData::Symbol { word: a, .. }, ValueData::Word(b)) => a == b,
            (
                ValueData::Symbol {
                    module: m1,
                    word: a,
                },
                ValueData::Symbol {
                    module: m2,
                    word: b,
                },
            ) => m1 == m2 && a == b,
            (ValueData::Macro, ValueData::Macro) => true,
            (ValueData::Literal(a), ValueData::Literal(b)) => a == b,
            (ValueData::EndOfInput, ValueData::EndOfInput) => true,
            _ => false,
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for ValueData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (ValueData::Integer(a), ValueData::Integer(b)) => a.partial_cmp(b),
            (ValueData::Float(a), ValueData::Float(b)) => a.partial_cmp(b),
            (ValueData::Integer(a), ValueData::Float(b)) => (*a as f64).partial_cmp(b),
            (ValueData::Float(a), ValueData::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (ValueData::Char(a), ValueData::Char(b)) => a.partial_cmp(b),
            (ValueData::Boolean(a), ValueData::Boolean(b)) => a.partial_cmp(b),
            (ValueData::List(a), ValueData::List(b)) => {
                // First compare lengths
                match a.len().partial_cmp(&b.len()) {
                    Some(Ordering::Equal) => {
                        // If lengths are equal, compare elements
                        for (x, y) in a.iter().zip(b.iter()) {
                            let x_val = &*x.borrow();
                            let y_val = &*y.borrow();
                            match x_val.partial_cmp(y_val) {
                                Some(Ordering::Equal) => continue,
                                other => return other,
                            }
                        }
                        Some(Ordering::Equal)
                    }
                    other => other,
                }
            }
            // TODO: compare hashmaps
            (ValueData::String(a), ValueData::String(b)) => a.partial_cmp(b),
            (ValueData::Function(a), ValueData::Function(b)) => a.partial_cmp(b), // Functions cannot be ordered
            (ValueData::Word(a), ValueData::Word(b)) => a.partial_cmp(b),
            (
                ValueData::Symbol {
                    module: m1,
                    word: w1,
                },
                ValueData::Symbol {
                    module: m2,
                    word: w2,
                },
            ) => (m1, w1).partial_cmp(&(m2, w2)),

            _ => None,
        }
    }
}

impl Hash for ValueData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ValueData::Integer(n) => n.hash(state),
            ValueData::Float(f) => f.to_bits().hash(state),
            ValueData::Boolean(b) => b.hash(state),
            ValueData::Char(c) => c.hash(state),
            ValueData::String(s) => s.hash(state),
            ValueData::List(list) => {
                for item in list {
                    item.borrow().hash(state);
                }
            }
            ValueData::HashMap(map) => {
                let mut map = map.iter().collect::<Vec<_>>();
                map.sort_by_key(|p| p.0);
                for (k, v) in map {
                    k.hash(state);
                    v.borrow().hash(state);
                }
            }
            ValueData::Function(lambda) => lambda.hash(state),
            ValueData::Address(addr) => addr.hash(state),
            ValueData::Word(word) => word.hash(state),
            ValueData::Symbol { module, word } => {
                module.hash(state);
                word.hash(state);
            }
            ValueData::Macro => "MACRO".hash(state),
            ValueData::Literal(value) => value.hash(state),
            ValueData::Return(address, breakpoint) => {
                address.hash(state);
                breakpoint.hash(state);
            }
            ValueData::Writer(writer) => writer.hash(state),
            ValueData::Reader(reader) => reader.hash(state),
            ValueData::EndOfInput => "EOI".hash(state),
        }
    }
}

impl Add for ValueData {
    type Output = Result<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => Ok(ValueData::Integer(a + b)),
            (ValueData::Float(a), ValueData::Float(b)) => Ok(ValueData::Float(a + b)),
            (ValueData::Integer(a), ValueData::Float(b)) => Ok(ValueData::Float(*a as f64 + b)),
            (ValueData::Float(a), ValueData::Integer(b)) => Ok(ValueData::Float(a + *b as f64)),
            _ => Err(VMError::TypeMismatch(format!("{:?} + {:?}", &self, &rhs)).into()),
        }
    }
}

impl Sub for ValueData {
    type Output = Result<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => Ok(ValueData::Integer(a - b)),
            (ValueData::Float(a), ValueData::Float(b)) => Ok(ValueData::Float(a - b)),
            (ValueData::Integer(a), ValueData::Float(b)) => Ok(ValueData::Float(a as f64 - b)),
            (ValueData::Float(a), ValueData::Integer(b)) => Ok(ValueData::Float(a - b as f64)),
            _ => Err(VMError::TypeMismatch("subtraction".to_string()).into()),
        }
    }
}

impl Mul for ValueData {
    type Output = Result<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => Ok(ValueData::Integer(a * b)),
            (ValueData::Float(a), ValueData::Float(b)) => Ok(ValueData::Float(a * b)),
            (ValueData::Integer(a), ValueData::Float(b)) => Ok(ValueData::Float(a as f64 * b)),
            (ValueData::Float(a), ValueData::Integer(b)) => Ok(ValueData::Float(a * b as f64)),
            _ => Err(VMError::TypeMismatch("multiplication".to_string()).into()),
        }
    }
}

impl Div for ValueData {
    type Output = Result<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => {
                if b == 0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Integer(a / b))
                }
            }
            (ValueData::Float(a), ValueData::Float(b)) => {
                if b == 0.0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Float(a / b))
                }
            }
            (ValueData::Integer(a), ValueData::Float(b)) => {
                if b == 0.0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Float(a as f64 / b))
                }
            }
            (ValueData::Float(a), ValueData::Integer(b)) => {
                if b == 0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Float(a / b as f64))
                }
            }
            _ => Err(VMError::TypeMismatch("division".to_string()).into()),
        }
    }
}
