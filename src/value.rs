use crate::error::{Error, Result};
use crate::parser::TokenType;
use std::convert::TryFrom;
use std::ops::{Add, Div, Mul, Sub};
use std::{fmt, result};

use num::Rational64;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Rational(Rational64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Rational(r) => write!(f, "{}/{}", r.numer(), r.denom()),
            Value::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl TryFrom<TokenType> for Value {
    type Error = Error;

    fn try_from(value: TokenType) -> result::Result<Self, Self::Error> {
        match value {
            TokenType::Integer(number) => Ok(Value::Integer(number)),
            TokenType::Float(number) => Ok(Value::Float(number)),
            TokenType::Rational(num, denom) => Ok(Value::Rational(Rational64::new(num, denom))),
            TokenType::String(string) => Ok(Value::String(string)),
            _ => Err(Error::TokenTypeNotValue(value)),
        }
    }
}

// TODO: revisit the clones from here on out
impl Value {
    pub fn checked_div(self, other: Value) -> Result<Value> {
        match (self.clone(), other.clone()) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(Error::DivideByZero)
                } else {
                    Ok(Value::Integer(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(Error::DivideByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Rational(a), Value::Rational(b)) => {
                if *b.numer() == 0 {
                    Err(Error::DivideByZero)
                } else {
                    Ok(Value::Rational(a.div(b)))
                }
            }
            _ => Err(Error::InvalidOperands(self.to_string(), other.to_string())),
        }
    }
}

impl Div for Value {
    type Output = Result<Value>;

    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(rhs)
    }
}

// TODO: long integers
// TODO: checked math
impl Add for Value {
    type Output = Result<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.clone(), rhs.clone()) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(Error::InvalidOperands(self.to_string(), rhs.to_string())),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self.clone(), rhs.clone()) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err(Error::InvalidOperands(self.to_string(), rhs.to_string())),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.clone(), rhs.clone()) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Rational(a), Value::Rational(b)) => Ok(Value::Rational(a * b)),
            _ => Err(Error::InvalidOperands(self.to_string(), rhs.to_string())),
        }
    }
}

#[cfg(test)]
mod tests;
