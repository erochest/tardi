use pretty_assertions::assert_eq;

use crate::error::Result;
use crate::token::{Token, TokenType};

use super::*;

fn test_compile(input: Vec<TokenType>, expected: Vec<u8>) {
    let tokens = input.into_iter().map(|tt| Token { token_type: tt, line_no: 1 }).collect();
    let chunk = compile(tokens);
    assert_eq!(chunk.code, expected);
}

#[test]
fn test_compile_simple_expression_with_div() {
    test_compile(vec!["10".try_into().unwrap(), "3".try_into().unwrap(), "/".try_into().unwrap()], vec![0, 0, 0, 1, OpCode::Div as u8]);
}

#[test]
fn test_compile_simple_expression() {
    test_compile(vec!["10".try_into().unwrap(), "3".try_into().unwrap()], vec![0, 0, 0, 1]);
}

#[test]
fn test_compile_simple_expression_with_add() {
    test_compile(vec!["10".try_into().unwrap(), "3".try_into().unwrap(), "+".try_into().unwrap()], vec![0, 0, 0, 1, OpCode::Add as u8]);
}

#[test]
fn test_compile_simple_expression_with_sub() {
    test_compile(vec!["10".try_into().unwrap(), "3".try_into().unwrap(), "-".try_into().unwrap()], vec![0, 0, 0, 1, OpCode::Sub as u8]);
}

#[test]
fn test_compile_simple_expression_with_mult() {
    test_compile(vec!["10".try_into().unwrap(), "3".try_into().unwrap(), "*".try_into().unwrap()], vec![0, 0, 0, 1, OpCode::Mult as u8]);
}

#[test]
fn test_compile_string() {
    test_compile(vec!["\"hello\"".try_into().unwrap()], vec![OpCode::GetConstant as u8, 0]);
}
