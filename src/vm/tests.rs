use pretty_assertions::assert_eq;

use crate::value::Value;

use super::*;

fn test_chunk(chunk: Chunk, expected: &[Value]) {
    let mut vm = VM::new();
    vm.execute(chunk).unwrap();
    assert_eq!(vm.stack, expected);
}

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

#[test]
fn test_execute_equals() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(4), Value::Integer(3)];
    chunk.code = vec![
        0,
        0,
        0,
        0,
        OpCode::Equal as u8,
        0,
        0,
        0,
        1,
        OpCode::Equal as u8,
    ];
    test_chunk(chunk, &[Value::Boolean(true), Value::Boolean(false)]);
}

#[test]
fn test_execute_not() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Boolean(false), Value::Boolean(true)];
    chunk.code = vec![0, 0, OpCode::Not as u8, 0, 1, OpCode::Not as u8];
    test_chunk(chunk, &[Value::Boolean(true), Value::Boolean(false)]);
}

#[test]
fn test_execute_less() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(4), Value::Integer(3)];
    chunk.code = vec![
        0,
        0,
        0,
        0,
        OpCode::Less as u8,
        0,
        1,
        0,
        0,
        OpCode::Less as u8,
    ];
    test_chunk(chunk, &[Value::Boolean(false), Value::Boolean(true)]);
}

#[test]
fn test_execute_greater() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(4), Value::Integer(3)];
    chunk.code = vec![
        0,
        0,
        0,
        1,
        OpCode::Greater as u8,
        0,
        1,
        0,
        0,
        OpCode::Greater as u8,
    ];
    test_chunk(chunk, &[Value::Boolean(true), Value::Boolean(false)]);
}
