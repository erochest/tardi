use std::cell::RefCell;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use lambda::Lambda;

use crate::error::{Error, Result, VMError};
use crate::shared::{shared, unshare_clone};
pub use crate::value::data::ValueData;
pub use crate::value::io::reader::TardiReader;
pub use crate::value::io::writer::TardiWriter;

pub mod data;
pub mod io;
pub mod lambda;

// TODO: group Value and ValueData implementations better

/// Shared value type for all values
pub type SharedValue = Rc<RefCell<Value>>;

#[derive(Debug, Clone, Eq)]
pub struct Value {
    pub data: ValueData,

    /// The actual text of the token from source
    pub lexeme: Option<String>,

    pub pos: Option<Pos>,
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
        // We don't include lexeme and pos in the hash calculation
        // because they don't affect equality in PartialEq
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Pos {
    /// Line number in source (1-based)
    pub line: usize,

    /// Column number in source (1-based)
    pub column: usize,

    /// Offset from start of source (0-based)
    pub offset: usize,

    /// Length of the token in characters
    pub length: usize,
}

impl Value {
    pub fn new(data: ValueData) -> Self {
        Value {
            data,
            lexeme: None,
            pos: None,
        }
    }

    pub fn with_lexeme(data: ValueData, lexeme: &str) -> Self {
        Value {
            data,
            lexeme: Some(lexeme.to_string()),
            pos: None,
        }
    }

    pub fn with_pos(data: ValueData, lexeme: &str, pos: Pos) -> Self {
        Value {
            data,
            lexeme: Some(lexeme.to_string()),
            pos: Some(pos),
        }
    }

    pub fn from_parts(
        data: ValueData,
        lexeme: &str,
        line: usize,
        column: usize,
        offset: usize,
        length: usize,
    ) -> Self {
        let pos = Pos {
            line,
            column,
            offset,
            length,
        };
        Value::with_pos(data, lexeme, pos)
    }

    pub fn to_repr(&self) -> String {
        if let ValueData::String(ref s) = self.data {
            format!("\"{}\"", s.replace("\\", "\\\\").replace("\"", "\\\""))
        } else {
            self.data.to_string()
        }
    }

    // TODO: change these into as_boolean
    pub fn as_boolean(&self) -> Option<bool> {
        if let ValueData::Boolean(b) = self.data {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let ValueData::Integer(i) = self.data {
            Some(i)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let ValueData::String(ref s) = self.data {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_char(&self) -> Option<char> {
        if let ValueData::Char(c) = self.data {
            Some(c)
        } else {
            None
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self.data, ValueData::List(_))
    }

    pub fn as_list(&self) -> Option<&Vec<SharedValue>> {
        if let ValueData::List(ref list) = self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<SharedValue>> {
        if let ValueData::List(ref mut list) = self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Lambda> {
        if let ValueData::Function(ref callable) = self.data {
            Some(callable)
        } else {
            None
        }
    }

    pub fn as_function_mut(&mut self) -> Option<&mut Lambda> {
        if let ValueData::Function(ref mut callable) = self.data {
            Some(callable)
        } else {
            None
        }
    }

    pub fn as_address(&self) -> Option<usize> {
        match self.data {
            ValueData::Address(address) => Some(address),
            ValueData::Return(address, _) => Some(address),
            _ => None,
        }
    }

    pub fn is_breakpoint(&self) -> bool {
        if let ValueData::Return(_, breakpoint) = self.data {
            breakpoint
        } else {
            false
        }
    }

    pub fn as_word(&self) -> Option<&str> {
        if let ValueData::Word(ref w) = self.data {
            Some(w)
        } else if let ValueData::Symbol { word: ref w, .. } = self.data {
            Some(w)
        } else {
            None
        }
    }

    pub fn as_symbol(&self) -> Option<(&str, &str)> {
        if let ValueData::Symbol {
            ref module,
            ref word,
        } = self.data
        {
            Some((module, word))
        } else {
            None
        }
    }
}

// From implementations for each of the contained values
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::with_lexeme(ValueData::Integer(value), &value.to_string())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::with_lexeme(ValueData::Float(value), &value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::with_lexeme(ValueData::Boolean(value), &value.to_string())
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::with_lexeme(ValueData::Char(value), &value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        let value = ValueData::String(value);
        let lexeme = format!("{}", value);
        Value::with_lexeme(value, &lexeme)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        let value = value.to_string();
        let value = ValueData::String(value);
        let lexeme = format!("{}", value);
        Value::with_lexeme(value, &lexeme)
    }
}

impl From<u8> for Value {
    fn from(byte: u8) -> Self {
        Value::from(byte as i64)
    }
}

impl From<i32> for Value {
    fn from(byte: i32) -> Self {
        Value::from(byte as i64)
    }
}

impl From<Vec<SharedValue>> for Value {
    fn from(value: Vec<SharedValue>) -> Self {
        let repr = value
            .iter()
            .map(|v| format!("{}", &v.borrow()))
            .collect::<Vec<_>>();
        let repr = format!("[ {} ]", repr.join(" "));
        Value::with_lexeme(ValueData::List(value), &repr)
    }
}

impl<V> From<Vec<V>> for Value
where
    V: Into<Value>,
{
    fn from(vector: Vec<V>) -> Self
    where
        V: Into<Value>,
    {
        vector
            .into_iter()
            .map(|v| shared(v.into()))
            .collect::<Vec<_>>()
            .into()
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value.data {
            ValueData::List(list) => Ok(list.into_iter().map(unshare_clone).collect()),
            _ => {
                Err(VMError::TypeMismatch(format!("cannot convert to list {}", value.data)).into())
            }
        }
    }
}

pub struct ValueVec<'a>(pub &'a Vec<Value>);

impl fmt::Display for ValueVec<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ValueVec(values) = self;
        write!(
            f,
            "{{ {} }}",
            values
                .iter()
                .map(|v| v.to_repr())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Add for Value {
    type Output = Result<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Ok(Value::new((self.data + rhs.data)?))
    }
}

impl Sub for Value {
    type Output = Result<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Ok(Value::new((self.data - rhs.data)?))
    }
}

impl Mul for Value {
    type Output = Result<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        Ok(Value::new(self.data.mul(rhs.data)?))
    }
}

impl Div for Value {
    type Output = Result<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        Ok(Value::new(self.data.div(rhs.data)?))
    }
}
