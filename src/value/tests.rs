use std::{convert::TryFrom, f64::consts::PI};

use super::*;

use crate::scanner::TokenType;

// TODO: may want to break this file up

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
fn test_add_float() {
    let result = Value::from(13.5) + 0.75.into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 14.25.into());
}

#[test]
fn test_sub_float() {
    let result = Value::from(13.5) - 0.75.into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 12.75.into());
}

#[test]
fn test_mul_float() {
    let result = Value::from(13.5) * 2.0.into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 27.0.into());
}

#[test]
fn test_div_happy_path_float() {
    let result = Value::from(13.5) / 2.0.into();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 6.75.into());
}

#[test]
fn test_div_by_zero_float() {
    let result = Value::from(13.5) / 0.0.into();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::DivideByZero));
}

#[test]
fn test_add_rational() {
    let result = Value::Rational(Rational64::new(1, 3)) + Value::Rational(Rational64::new(1, 3));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Rational(Rational64::new(2, 3)));
}

#[test]
fn test_add_rational_different_denominators() {
    let result = Value::Rational(Rational64::new(1, 5)) + Value::Rational(Rational64::new(1, 3));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Rational(Rational64::new(8, 15)));
}

#[test]
fn test_sub_rational() {
    let result = Value::Rational(Rational64::new(1, 3)) - Value::Rational(Rational64::new(2, 3));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Rational(-Rational64::new(1, 3)));
}

#[test]
fn test_sub_rational_different_denominators() {
    let result = Value::Rational(Rational64::new(1, 3)) - Value::Rational(Rational64::new(1, 5));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Rational(Rational64::new(2, 15)));
}

#[test]
fn test_mul_rational() {
    let result = Value::Rational(Rational64::new(1, 3)) * Value::Rational(Rational64::new(1, 4));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Rational(Rational64::new(1, 12)));
}

#[test]
fn test_div_rational() {
    let result = Value::Rational(Rational64::new(1, 12)) / Value::Rational(Rational64::new(3, 1));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Rational(Rational64::new(1, 36)));
}

#[test]
fn test_div_rational_zero_denominator() {
    let result = Value::Rational(Rational64::new(1, 3)) / Value::Rational(Rational64::new(0, 3));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::DivideByZero));
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
fn test_from_usize() {
    let result = Value::from(74usize);
    assert_eq!(result, Value::Address(74));
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
fn test_try_from_tokentype_rational() {
    let result = Value::try_from(TokenType::Rational(1, 4));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Rational(Rational64::new(1, 4)));
}

#[test]
fn test_try_from_tokentype_string() {
    let result = Value::try_from(TokenType::String("hello world".to_string()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
}

#[test]
fn test_try_from_tokentype_boolean() {
    let result = Value::try_from(TokenType::Boolean(false));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_display_vector() {
    let value: Value = vec![Value::from(42i64)].into();
    assert_eq!("{ 42 }", format!("{}", value));
}

#[test]
#[ignore = "value sharing"]
fn test_add_vectors() {
    let a: Value = vec![Value::from(2i64)].into();
    let b: Value = vec![Value::from(3i64)].into();

    let c = a + b;

    assert!(c.is_ok());
    let c = c.unwrap();
    assert_eq!(Value::from(vec![Value::from(2i64), Value::from(3i64)]), c);
}

#[test]
fn test_mod_operand_error() {
    let a = Value::from(6i64);
    let b = Value::from(3.0);

    let c = a % b;

    assert!(c.is_err_and(|e| matches!(e, Error::InvalidOperands(_, _))));
}

#[test]
fn test_mod_integers() {
    let a = Value::from(6i64);
    let b = Value::from(3i64);

    let c = a % b;

    assert!(c.is_ok());
    assert_eq!(Value::from(0i64), c.unwrap());
}

#[test]
fn test_mod_floats() {
    let a = Value::from(6.0);
    let b = Value::from(3.0);

    let c = a % b;

    assert!(c.is_ok());
    assert_eq!(Value::from(0.0), c.unwrap());
}

#[test]
fn test_mod_rationals() {
    let a = Value::Rational(Rational64::new(1, 4));
    let b = Value::Rational(Rational64::new(1, 2));

    let c = a % b;

    assert!(c.is_ok());
    assert_eq!(Value::Rational(Rational64::new(1, 4)), c.unwrap());
}

#[test]
fn test_address_arithmetic() {
    // Test Address + Integer
    let address = Value::Address(5);
    let integer = Value::Integer(3);
    assert_eq!(Value::Address(8), (address.clone() + integer.clone()).unwrap());

    // Test Integer + Address
    assert_eq!(Value::Address(8), (integer.clone() + address.clone()).unwrap());

    // Test Address - Integer
    assert_eq!(Value::Address(2), (address - integer).unwrap());

    // Test Address + Integer (overflow case)
    let max_address = Value::Address(usize::MAX);
    let one = Value::Integer(1);
    assert!(matches!((max_address + one), Err(Error::MathOverflow)));

    // Test Address - Integer (underflow case)
    let zero_address = Value::Address(0);
    let one = Value::Integer(1);
    assert!(matches!((zero_address - one), Err(Error::MathUnderflow)));
}

#[test]
fn test_display_floats() {
    let zero = Value::from(0.0);
    let output = format!("{}", zero);
    assert_eq!("0.0", &output);

    let quarter = Value::from(0.25);
    let output = format!("{}", quarter);
    assert_eq!("0.25", &output);
}
