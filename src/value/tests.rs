use std::{convert::TryFrom, f64::consts::PI};

use super::*;

use crate::parser::TokenType;

#[test]
fn test_add_integer() {
    let result = Value::Integer(1) + Value::Integer(2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(3));
}

#[test]
fn test_sub_integer() {
    let result = Value::Integer(1) - Value::Integer(2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(-1));
}

#[test]
fn test_mul_integer() {
    let result = Value::Integer(1) * Value::Integer(2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(2));
}

#[test]
fn test_checked_div_by_zero() {
    let result = Value::Integer(4).checked_div(Value::Integer(0));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::DivideByZero));
}

#[test]
fn test_checked_div_not_number() {
    let result = Value::String("1".to_string()).checked_div(Value::String("2".to_string()));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidOperands(_, _)));
}

#[test]
fn test_checked_div_happy_path() {
    let result = Value::Integer(4).checked_div(Value::Integer(2));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(2));
}

#[test]
fn test_div_integer() {
    let result = Value::Integer(4) / Value::Integer(2);
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
    let result = Value::from(PI);
    assert_eq!(result, Value::Float(PI));
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
