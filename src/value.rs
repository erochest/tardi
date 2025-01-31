use std::fmt;
use std::ops::{Add, Mul, Sub};
use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
        }
    }
}

impl Add for Value {
    type Output = Result<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (a, b) => Err(Error::InvalidOperands(a.to_string(), b.to_string())),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (a, b) => Err(Error::InvalidOperands(a.to_string(), b.to_string())),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (a, b) => Err(Error::InvalidOperands(a.to_string(), b.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let result = Value::Integer(1) + Value::Integer(2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_sub() {
        let result = Value::Integer(1) - Value::Integer(2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(-1));
    }

    #[test]
    fn test_mul() {
        let result = Value::Integer(1) * Value::Integer(2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }
}
