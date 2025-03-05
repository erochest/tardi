use crate::op_code::OpCode;

use super::*;

#[test]
fn test_add_constant() {
    let mut chunk = Chunk::new();
    let constant = Value::Integer(10);
    let index = chunk.add_constant(constant.clone());
    assert_eq!(index, 0);
    assert_eq!(chunk.constants.len(), 1);
    assert_eq!(chunk.constants[0], constant);
}

#[test]
fn test_push_opcode() {
    let mut chunk = Chunk::new();
    let constant = Value::Integer(10);
    let index = chunk.add_constant(constant.clone());

    chunk.push_op_code(OpCode::GetConstant, index as u8);

    assert_eq!(chunk.code.len(), 2);
    assert_eq!(chunk.code, vec![0, 0]);
}

#[test]
fn test_define_stack_ops() {
    let chunk = Chunk::new();
    assert!(chunk.builtin_index.contains_key("drop"));
    assert!(chunk.builtin_index.contains_key("dup"));
    assert!(chunk.builtin_index.contains_key("nip"));
    assert!(chunk.builtin_index.contains_key("over"));
    assert!(chunk.builtin_index.contains_key("pop"));
    assert!(chunk.builtin_index.contains_key("rot"));
    assert!(chunk.builtin_index.contains_key("swap"));
}
