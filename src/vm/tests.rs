use pretty_assertions::assert_eq;

use super::*;

fn test_chunk(chunk: Chunk, expected: &[Value]) {
    let mut vm = VM::new();
    vm.execute(chunk).unwrap();
    assert_eq!(vm.stack, expected);
}

use crate::compiler::compile;
use crate::parser::parse;
use crate::value::Value;

#[test]
fn test_execute_get_constant() {
    // AI: constants: Value::Integer(1)
    // AI: code: 0, 0
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(1)];
    chunk.code = vec![0, 0];
    test_chunk(chunk, &[Value::Integer(1)]);
}

#[test]
fn test_execute_add() {
    // AI: constants: Value::Integer(1), Value::Integer(2)
    // AI: code: 0, 0, 0, 1, OpCode::Add
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(1), Value::Integer(2)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Add as u8];
    test_chunk(chunk, &[Value::Integer(3)]);
}

#[test]
fn test_execute_sub() {
    // AI: constants: Value::Integer(2), Value::Integer(1)
    // AI: code: 0, 0, 0, 1, OpCode::Sub
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(2), Value::Integer(1)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Sub as u8];
    test_chunk(chunk, &[Value::Integer(1)]);
}

#[test]
fn test_execute_mult() {
    // AI: constants: Value::Integer(2), Value::Integer(3)
    // AI: code: 0, 0, 0, 1, OpCode::Mult
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(2), Value::Integer(3)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Mult as u8];
    test_chunk(chunk, &[Value::Integer(6)]);
}

#[test]
fn test_execute_div() {
    // AI: constants: Value::Integer(6), Value::Integer(3)
    // AI: code: 0, 0, 0, 1, OpCode::Div
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(6), Value::Integer(3)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Div as u8];
    test_chunk(chunk, &[Value::Integer(2)]);
}
