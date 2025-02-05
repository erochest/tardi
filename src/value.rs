use std::fmt;
use std::ops::{Add, Mul, Sub};
use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    String(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl Value {
    pub fn checked_div(self, other: Value) -> Result<Value> {
        match (self.clone(), other.clone()) {
            (Value::Integer(a), Value::Integer(b)) => {
                if b == 0 {
                    Err(Error::DivideByZero)
                } else {
                    Ok(Value::Integer(a / b))
                }
            },
            (Value::String(_), Value::String(_)) => {
                Err(Error::InvalidOperands(self.to_string(), other.to_string()))
            },
            _ => Err(Error::InvalidOperands(self.to_string(), other.to_string())),
        }
    }
}

impl Add for Value {
    type Output = Result<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.clone(), rhs.clone()) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
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
            _ => Err(Error::InvalidOperands(self.to_string(), rhs.to_string())),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.clone(), rhs.clone()) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            _ => Err(Error::InvalidOperands(self.to_string(), rhs.to_string())),
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
