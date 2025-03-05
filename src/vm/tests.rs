use pretty_assertions::assert_eq;

use crate::value::Value;

use super::*;

fn test_chunk(chunk: &mut Chunk, expected: &[Value]) {
    let mut vm = VM::new();
    let result = vm.execute(chunk);
    assert!(result.is_ok(), "VM error on {:?}", result);
    let stack = vm
        .stack
        .iter()
        .map(|x| {
            let x = x.borrow();
            let x = x.clone();
            x
        })
        .collect::<Vec<_>>();
    assert_eq!(stack, expected);
}

#[test]
fn test_execute_get_constant() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(1)];
    chunk.code = vec![0, 0];
    test_chunk(&mut chunk, &[Value::Integer(1)]);
}

#[test]
fn test_execute_add() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(1), Value::Integer(2)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Add as u8];
    test_chunk(&mut chunk, &[Value::Integer(3)]);
}

#[test]
fn test_execute_sub() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(2), Value::Integer(1)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Sub as u8];
    test_chunk(&mut chunk, &[Value::Integer(1)]);
}

#[test]
fn test_execute_mult() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(2), Value::Integer(3)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Mult as u8];
    test_chunk(&mut chunk, &[Value::Integer(6)]);
}

#[test]
fn test_execute_div() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Integer(6), Value::Integer(3)];
    chunk.code = vec![0, 0, 0, 1, OpCode::Div as u8];
    test_chunk(&mut chunk, &[Value::Integer(2)]);
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
    test_chunk(&mut chunk, &[Value::Boolean(true), Value::Boolean(false)]);
}

#[test]
fn test_execute_not() {
    let mut chunk = Chunk::new();
    chunk.constants = vec![Value::Boolean(false), Value::Boolean(true)];
    chunk.code = vec![0, 0, OpCode::Not as u8, 0, 1, OpCode::Not as u8];
    test_chunk(&mut chunk, &[Value::Boolean(true), Value::Boolean(false)]);
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
    test_chunk(&mut chunk, &[Value::Boolean(false), Value::Boolean(true)]);
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
    test_chunk(&mut chunk, &[Value::Boolean(true), Value::Boolean(false)]);
}

#[test]
fn test_execute_jump() {
    // env_logger::builder().init();
    let mut chunk = Chunk::new();
    // : double ( x -- y ) 2 * ;
    // 4 double
    // 5 double
    chunk.constants = vec![Value::Integer(2), 4i64.into(), 5i64.into()];
    chunk.code = vec![
        OpCode::Jump as u8,
        6,
        OpCode::GetConstant as u8,
        0,
        OpCode::Mult as u8,
        OpCode::Return as u8,
        OpCode::GetConstant as u8,
        1,
        OpCode::MarkJump as u8,
        2,
        OpCode::GetConstant as u8,
        2,
        OpCode::MarkJump as u8,
        2,
        OpCode::Return as u8,
    ];
    test_chunk(&mut chunk, &[Value::Integer(8), 10i64.into()]);
}

#[test]
fn test_execute_lambda_call() {
    // env_logger::builder().init();
    let mut chunk = Chunk::new();
    // let input = "4 [ 2 * ] call";
    chunk.constants = vec![
        Value::Integer(4),
        2i64.into(),
        Value::Lambda("[ 2 * ]".to_string(), 4),
    ];
    chunk.code = vec![
        OpCode::GetConstant as u8,
        0,
        OpCode::Jump as u8,
        8,
        OpCode::GetConstant as u8,
        1,
        OpCode::Mult as u8,
        OpCode::Return as u8,
        OpCode::GetConstant as u8,
        2,
        OpCode::CallTardiFn as u8,
        0,
        OpCode::Return as u8,
    ];
    test_chunk(&mut chunk, &[8i64.into()]);
}

#[test]
fn test_execute_call_stack_ops() {
    let mut chunk = Chunk::new();
    let pop_index = chunk.builtin_index["pop"];
    chunk.constants = vec![Value::Integer(4), 8i64.into(), 16i64.into()];
    chunk.code = vec![
        OpCode::GetConstant as u8,
        0,
        OpCode::GetConstant as u8,
        1,
        OpCode::ToCallStack as u8,
        OpCode::CallTardiFn as u8,
        pop_index as u8,
        OpCode::CopyCallStack as u8,
        OpCode::FromCallStack as u8,
        OpCode::Return as u8,
    ];
    test_chunk(&mut chunk, &[8i64.into(), 8i64.into()]);
}
