use crate::error::{Error, Result};
use crate::scanner::{Token, TokenType};
use std::convert::TryFrom;
use std::ops::{Add, Div, Mul, Sub};
use std::{fmt, result};

use num::Rational64;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Rational(Rational64),
    Boolean(bool),
    String(String),
    Vector(Vec<Value>),
    Function(Function),
    Lambda(String, usize),
    Address(usize),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Rational(r) => write!(f, "{}/{}", r.numer(), r.denom()),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Vector(values) => {
                write!(f, "{{ ")?;
                for item in values {
                    write!(f, "{} ", item)?;
                }
                write!(f, "}}")
            }
            Value::Function(function) => {
                write!(f, "{}", function)
            }
            Value::Lambda(repr, loc) => write!(f, "{}", repr),
            Value::Address(addr) => write!(f, "<@{}>", addr),
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

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Address(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        let values = value.into_iter().collect::<Vec<_>>();
        Value::Vector(values)
    }
}

impl TryFrom<Token> for Value {
    type Error = Error;

    fn try_from(token: Token) -> result::Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Integer(i) => Ok(Value::Integer(i)),
            TokenType::Float(f) => Ok(Value::Float(f)),
            TokenType::Rational(n, d) => Ok(Value::Rational(Rational64::new(n, d))),
            TokenType::String(s) => Ok(Value::String(s)),
            TokenType::Boolean(b) => Ok(Value::Boolean(b)),
            _ => Err(Error::InvalidToken(format!("{:?}", token))),
        }
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
            TokenType::Boolean(b) => Ok(Value::Boolean(b)),
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
            (Value::Vector(_a), Value::Vector(_b)) => {
                // let c = Vec::with_capacity(a.len() + b.len());
                // TODO: `Vec::append` moves everything from a and b into c.
                // I probably need to work out sharing and who owns what now.
                // I probably need to think more clearly about this and have
                // a plan or a framework.
                todo!("vector addition (and value sharing)")
            }
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

// TODO: need to keep the body here for printing later.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Function {
    pub name: String,
    pub doc_comment: Option<String>,
    pub type_declaration: TypeDeclaration,
    pub ip: u8,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TypeDeclaration {
    inputs: Vec<String>,
    outputs: Vec<String>,
}

impl TypeDeclaration {
    pub fn new(inputs: Vec<String>, outputs: Vec<String>) -> Self {
        Self { inputs, outputs }
    }
}

impl fmt::Display for TypeDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[cfg(test)]
mod tests;
