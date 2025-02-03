use pretty_assertions::assert_eq;

use super::*;

use crate::chunk::Chunk;
use crate::compiler::compile;
use crate::parser::parse;
use crate::value::Value;



#[test]
fn test_execute_get_constant() {
    let input = "1";
    let tokens = parse(input).into_iter().collect::<Result<Vec<_>>>().unwrap();
    let chunk = compile(tokens);

    let mut vm = VM::new();
    vm.execute(chunk).unwrap();

    assert_eq!(vm.stack, vec![Value::Integer(1)]);
}

#[test]
fn test_execute_add() {
    let input = "1 2 +";
    let tokens = parse(input).into_iter().collect::<Result<Vec<_>>>().unwrap();
    let chunk = compile(tokens);

    let mut vm = VM::new();
    vm.execute(chunk).unwrap();

    assert_eq!(vm.stack, vec![Value::Integer(3)]);
}

#[test]
fn test_execute_sub() {
    let input = "2 1 -";
    let tokens = parse(input).into_iter().collect::<Result<Vec<_>>>().unwrap();
    let chunk = compile(tokens);

    let mut vm = VM::new();
    vm.execute(chunk).unwrap();

    assert_eq!(vm.stack, vec![Value::Integer(1)]);
}

#[test]
fn test_execute_mult() {
    let input = "2 3 *";
    let tokens = parse(input).into_iter().collect::<Result<Vec<_>>>().unwrap();
    let chunk = compile(tokens);

    let mut vm = VM::new();
    vm.execute(chunk).unwrap();

    assert_eq!(vm.stack, vec![Value::Integer(6)]);
}

#[test]
fn test_execute_div() {
    let input = "6 3 /";
    let tokens = parse(input).into_iter().collect::<Result<Vec<_>>>().unwrap();
    let chunk = compile(tokens);

    let mut vm = VM::new();
    vm.execute(chunk).unwrap();

    assert_eq!(vm.stack, vec![Value::Integer(2)]);
}
