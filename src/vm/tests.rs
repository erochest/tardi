use pretty_assertions::assert_eq;

use super::*;

fn test_chunk(chunk: Chunk, expected: &[Value]) {
    let mut vm = VM::new();
    vm.execute(chunk).unwrap();
    assert_eq!(vm.stack, expected);
}

use crate::value::Value;

#[test]
fn test_execute_get_constant() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(1)];
    chunk.code = vec![0, 0];
    test_chunk(chunk, &[Value::Integer(1)]);
}

#[test]
fn test_execute_add() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(1), Value::Integer(2)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Add as u8];
    test_chunk(chunk, &[Value::Integer(3)]);
}

#[test]
fn test_execute_sub() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(2), Value::Integer(1)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Sub as u8];
    test_chunk(chunk, &[Value::Integer(1)]);
}

#[test]
fn test_execute_mult() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(2), Value::Integer(3)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Mult as u8];
    test_chunk(chunk, &[Value::Integer(6)]);
}

#[test]
fn test_execute_div() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(6), Value::Integer(3)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Div as u8];
    test_chunk(chunk, &[Value::Integer(2)]);
}
