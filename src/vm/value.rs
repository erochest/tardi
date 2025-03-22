use std::cell::RefCell;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use crate::error::{Result, VMError};

/// Shared value type for all values
pub type SharedValue = Rc<RefCell<Value>>;

/// Helper function to create a SharedValue
pub fn shared(value: Value) -> SharedValue {
    Rc::new(RefCell::new(value))
}

/// Enum representing different types of values that can be stored on the stack
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),
    List(Vec<SharedValue>),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(x) => {
                let s = format!("{}", x);
                if !s.contains('.') {
                    write!(f, "{}.0", s)
                } else {
                    write!(f, "{}", s)
                }
            }
            Value::Boolean(true) => write!(f, "#t"),
            Value::Boolean(false) => write!(f, "#f"),
            Value::Char(c) => match c {
                '\n' => write!(f, "'\\n'"),
                '\r' => write!(f, "'\\r'"),
                '\t' => write!(f, "'\\t'"),
                '\\' => write!(f, "'\\\\'"),
                '\'' => write!(f, "'\\''"),
                c => write!(f, "'{}'", c),
            },
            Value::List(list) => {
                write!(f, "[")?;
                for item in list.iter() {
                    write!(f, " {}", item.borrow())?;
                }
                write!(f, " ]")
            }
            Value::String(s) => write!(f, "\"{}\"", s.replace('"', "\\\"")),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Integer(b)) => *a == (*b as f64),
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| &*x.borrow() == &*y.borrow())
            }
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Integer(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
            (Value::Float(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (Value::Char(a), Value::Char(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::List(a), Value::List(b)) => {
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
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Add for Value {
    type Output = Result<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + b as f64)),
            _ => Err(VMError::TypeMismatch("addition".to_string()).into()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - b as f64)),
            _ => Err(VMError::TypeMismatch("subtraction".to_string()).into()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * b as f64)),
            _ => Err(VMError::TypeMismatch("multiplication".to_string()).into()),
        }
    }
}

impl Div for Value {
    type Output = Result<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Integer(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(Value::Float(a as f64 / b))
                }
            }
            (Value::Float(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(Value::Float(a / b as f64))
                }
            }
            _ => Err(VMError::TypeMismatch("division".to_string()).into()),
        }
    }
}
