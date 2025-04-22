use std::cell::RefCell;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::ptr;
use std::rc::Rc;

use lambda::Lambda;

use crate::error::{Error, Result, VMError};
use crate::shared::{shared, unshare_clone};
use crate::{Compiler, Scanner};

pub mod lambda;

/// Shared value type for all values
pub type SharedValue = Rc<RefCell<Value>>;

#[derive(Debug, Clone, PartialOrd)]
pub struct Value {
    pub data: ValueData,

    // TODO: make this optional
    /// The actual text of the token from source
    pub lexeme: Option<String>,

    pub pos: Option<Pos>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

// -- they're more closely tied to `Environment` and they're part of what
// bridges across layers
/// Enum representing different types of values that can be stored on the stack
#[derive(Debug, Clone)]
pub enum ValueData {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),
    String(String),
    List(Vec<SharedValue>),
    Function(Lambda),
    Address(usize),
    Word(String),
    Macro,
    Literal(Box<Value>),
    EndOfInput,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

    pub fn get_integer(&self) -> Option<i64> {
        if let ValueData::Integer(i) = self.data {
            Some(i)
        } else {
            None
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        if let ValueData::String(ref s) = self.data {
            Some(s)
        } else {
            None
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self.data, ValueData::List(_))
    }

    pub fn get_list(&self) -> Option<&Vec<SharedValue>> {
        if let ValueData::List(ref list) = self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn get_list_mut(&mut self) -> Option<&mut Vec<SharedValue>> {
        if let ValueData::List(ref mut list) = self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn get_function(&self) -> Option<&Lambda> {
        if let ValueData::Function(ref callable) = self.data {
            Some(callable)
        } else {
            None
        }
    }

    pub fn get_function_mut(&mut self) -> Option<&mut Lambda> {
        if let ValueData::Function(ref mut callable) = self.data {
            Some(callable)
        } else {
            None
        }
    }

    pub fn get_address(&self) -> Option<usize> {
        if let ValueData::Address(address) = self.data {
            Some(address)
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

impl From<ValueData> for Value {
    fn from(value: ValueData) -> Self {
        Value::new(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        ValueData::List(value.into_iter().map(shared).collect()).into()
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
            "[ {} ]",
            values
                .iter()
                .map(|v| format!("{}", v))
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
                write!(f, "[")?;
                for item in list.iter() {
                    write!(f, " {}", item.borrow())?;
                }
                write!(f, " ]")
            }
            ValueData::String(s) => write!(f, "\"{}\"", s.replace('"', "\\\"")),
            ValueData::Function(lambda) => write!(f, "{}", lambda),
            ValueData::Address(addr) => write!(f, "<@{}>", addr),
            ValueData::Word(word) => write!(f, "{}", word),
            ValueData::Macro => write!(f, "MACRO:"),
            ValueData::Literal(value) => write!(f, "\\ {}", value),
            ValueData::EndOfInput => write!(f, "<EOI>"),
        }
    }
}

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
            (ValueData::Function(a), ValueData::Function(b)) => a == b,
            (ValueData::Address(a), ValueData::Address(b)) => a == b,
            (ValueData::Word(a), ValueData::Word(b)) => a == b,
            (ValueData::Macro, ValueData::Macro) => true,
            (ValueData::Literal(a), ValueData::Literal(b)) => a == b,
            (ValueData::EndOfInput, ValueData::EndOfInput) => true,
            _ => false,
        }
    }
}

impl PartialOrd for ValueData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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
                    Some(std::cmp::Ordering::Equal) => {
                        // If lengths are equal, compare elements
                        for (x, y) in a.iter().zip(b.iter()) {
                            let x_val = &*x.borrow();
                            let y_val = &*y.borrow();
                            match x_val.partial_cmp(y_val) {
                                Some(std::cmp::Ordering::Equal) => continue,
                                other => return other,
                            }
                        }
                        Some(std::cmp::Ordering::Equal)
                    }
                    other => other,
                }
            }
            (ValueData::String(a), ValueData::String(b)) => a.partial_cmp(b),
            (ValueData::Function(a), ValueData::Function(b)) => a.partial_cmp(b), // Functions cannot be ordered
            (ValueData::Word(a), ValueData::Word(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Add for Value {
    type Output = Result<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Ok(Value::new((self.data + rhs.data)?))
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

impl Sub for Value {
    type Output = Result<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Ok(Value::new((self.data - rhs.data)?))
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

impl Mul for Value {
    type Output = Result<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        Ok(Value::new(self.data.mul(rhs.data)?))
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

impl Div for Value {
    type Output = Result<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        Ok(Value::new(self.data.div(rhs.data)?))
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
