use std::f64::consts::PI;

use pretty_assertions::assert_eq;

use crate::parser::{Token, TokenType};

use super::*;

fn test_compile(input: Vec<TokenType>, expected: Vec<u8>) {
    let tokens = input
        .into_iter()
        .map(|tt| Token {
            token_type: tt,
            line_no: 1,
            column: 0,
            length: 3,
        })
        .collect();
    let chunk = compile(tokens);
    assert_eq!(chunk.code, expected);
}

#[test]
fn test_compile_simple_expression_with_div() {
    test_compile(
        vec![10.into(), 3.into(), "/".parse().unwrap()],
        vec![0, 0, 0, 1, OpCode::Div as u8],
    );
}

#[test]
fn test_compile_simple_expression() {
    test_compile(vec![10.into(), 3.into()], vec![0, 0, 0, 1]);
}

#[test]
fn test_compile_simple_expression_with_add() {
    test_compile(
        vec![10.into(), 3.into(), "+".parse().unwrap()],
        vec![0, 0, 0, 1, OpCode::Add as u8],
    );
}

#[test]
fn test_compile_simple_expression_with_sub() {
    test_compile(
        vec![10.into(), 3.into(), "-".parse().unwrap()],
        vec![0, 0, 0, 1, OpCode::Sub as u8],
    );
}

#[test]
fn test_compile_simple_expression_with_mult() {
    test_compile(
        vec![10.into(), 3.into(), "*".parse().unwrap()],
        vec![0, 0, 0, 1, OpCode::Mult as u8],
    );
}

#[test]
fn test_compile_string() {
    test_compile(
        vec!["\"hello\"".parse().unwrap()],
        vec![OpCode::GetConstant as u8, 0],
    );
}

#[test]
fn test_compile_float() {
    test_compile(vec![PI.into()], vec![OpCode::GetConstant as u8, 0]);
}

#[test]
fn test_compile_rational() {
    test_compile(
        vec![TokenType::Rational(7, 9)],
        vec![OpCode::GetConstant as u8, 0],
    );
}

#[test]
fn test_compile_boolean() {
    test_compile(
        vec![TokenType::Boolean(true)],
        vec![OpCode::GetConstant as u8, 0],
    );
}

#[test]
fn test_compile_boolean_operators() {
    let input = vec![
        TokenType::Equal,
        TokenType::BangEqual,
        TokenType::Less,
        TokenType::LessEqual,
        TokenType::Greater,
        TokenType::GreaterEqual,
        TokenType::Bang,
    ];
    let expected = vec![
        OpCode::Equal as u8,
        OpCode::Equal as u8,
        OpCode::Not as u8,
        OpCode::Less as u8,
        OpCode::Greater as u8,
        OpCode::Not as u8,
        OpCode::Greater as u8,
        OpCode::Less as u8,
        OpCode::Not as u8,
        OpCode::Not as u8,
    ];
    test_compile(input, expected);
}
