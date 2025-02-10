use std::convert::TryFrom;

use super::*;

use crate::parser::TokenType;

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

#[test]
fn test_from_i64() {
    let result = Value::from(42_i64);
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_from_f64() {
    let result = Value::from(3.14_f64);
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn test_from_string() {
    let result = Value::from("hello".to_string());
    assert_eq!(result, Value::String("hello".to_string()));
}

#[test]
fn test_try_from_tokentype_integer() {
    let result = Value::try_from(TokenType::Integer(64));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(64));
}

#[test]
fn test_try_from_tokentype_float() {
    let result = Value::try_from(TokenType::Float(64.));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Float(64.));
}

#[test]
fn test_try_from_tokentype_string() {
    let result = Value::try_from(TokenType::String("hello world".to_string()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
}
