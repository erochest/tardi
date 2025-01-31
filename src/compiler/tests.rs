use pretty_assertions::assert_eq;

use crate::error::Result;
use crate::parser::parse;

use super::*;

fn test_compile(input: &str, expected: Vec<u8>) {
    let tokens = parse(input).into_iter().collect::<Result<Vec<_>>>().unwrap();
    let chunk = compile(tokens);
    assert_eq!(chunk.code, expected);
}

#[test]
fn test_compile_simple_expression() {
    test_compile("10 3", vec![0, 0, 0, 1]);
}

#[test]
fn test_compile_simple_expression_with_add() {
    test_compile("10 3 +", vec![0, 0, 0, 1, OpCode::Add as u8]);
}

#[test]
fn test_compile_simple_expression_with_sub() {
    test_compile("10 3 -", vec![0, 0, 0, 1, OpCode::Sub as u8]);
}

#[test]
fn test_compile_simple_expression_with_mult() {
    test_compile("10 3 *", vec![0, 0, 0, 1, OpCode::Mult as u8]);
}
