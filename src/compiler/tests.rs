use std::convert::TryFrom;

use super::*;
use crate::{Environment, Tardi};

use pretty_assertions::assert_eq;

// TODO: more tests

fn compile(input: &str) -> Result<Shared<Environment>> {
    let mut tardi = Tardi::default();
    let result = tardi.execute_str(input);
    result.map(|_| tardi.environment.clone())
}

fn get_ops(environment: Shared<Environment>) -> Vec<OpCode> {
    let mut actual_ops = Vec::new();
    let environment = environment.borrow();
    let instructions = environment.get_instructions();
    let mut i = 0;

    while i < instructions.len() {
        let op = instructions[i];
        let op = OpCode::try_from(op);
        assert!(op.is_ok(), "error: {:?}", op);
        let op = op.unwrap();
        actual_ops.push(op);
        if op == OpCode::Lit {
            i += 2;
        } else {
            i += 1;
        }
    }

    actual_ops
}

#[test]
fn test_compile_comparison_operators() -> Result<()> {
    let environment = compile("1 2 == 3 4 != 5 6 < 7 8 > 9 10 <= 11 12 >=")?;

    let expected_ops = vec![
        OpCode::Lit,
        OpCode::Lit,
        OpCode::Equal,
        OpCode::Lit,
        OpCode::Lit,
        OpCode::Equal,
        OpCode::Not,
        OpCode::Lit,
        OpCode::Lit,
        OpCode::Less,
        OpCode::Lit,
        OpCode::Lit,
        OpCode::Greater,
        OpCode::Lit,
        OpCode::Lit,
        OpCode::Greater,
        OpCode::Not,
        OpCode::Lit,
        OpCode::Lit,
        OpCode::Less,
        OpCode::Not,
        OpCode::Return,
    ];
    let actual_ops = get_ops(environment);

    assert_eq!(actual_ops, expected_ops);

    Ok(())
}

#[test]
fn test_compile_return_stack_operations() -> Result<()> {
    let environment = compile("42 >r r@ r>")?;

    let expected_ops = vec![
        OpCode::Lit,
        OpCode::ToR,
        OpCode::RFetch,
        OpCode::RFrom,
        OpCode::Return,
    ];
    let actual_ops = get_ops(environment);

    assert_eq!(actual_ops, expected_ops);
    Ok(())
}

#[test]
fn test_compile_word() -> Result<()> {
    let result = compile("custom_word");
    assert!(result.is_err());
    if let Err(Error::CompilerError(CompilerError::UndefinedWord(msg))) = result {
        assert_eq!(msg, "custom_word");
    } else {
        panic!("Expected UnsupportedToken error");
    }
    Ok(())
}

#[test]
fn test_compile_character_literals() -> Result<()> {
    let environment = compile("'a' '\\n' '\\t' '\\r' '\\'' '\\\\' '🦀' '\\u41' '\\u{1F600}'")?;

    let mut expected_ops = vec![OpCode::Lit; 9]; // One lit operation for each character
    expected_ops.push(OpCode::Return);

    let mut actual_ops = Vec::new();
    let environment = environment.borrow();
    let instructions = environment.get_instructions();
    let mut i = 0;
    while i < instructions.len() {
        let op = instructions[i];
        let op = OpCode::try_from(op).unwrap();
        actual_ops.push(op);
        if op == OpCode::Lit {
            // Verify the constant values
            let const_index = instructions[i + 1];
            let constant = environment.get_constant(const_index).unwrap();
            match (i / 2, constant) {
                (0, Value::Char('a')) => (),
                (1, Value::Char('\n')) => (),
                (2, Value::Char('\t')) => (),
                (3, Value::Char('\r')) => (),
                (4, Value::Char('\'')) => (),
                (5, Value::Char('\\')) => (),
                (6, Value::Char('🦀')) => (),
                (7, Value::Char('A')) => (),  // '\u41'
                (8, Value::Char('😀')) => (), // '\u{1F600}'
                _ => panic!("Unexpected constant at index {}: {:?}", i / 2, constant),
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    assert_eq!(actual_ops, expected_ops);
    Ok(())
}

#[ignore = "while i work out the details"]
#[test]
fn test_compile_macro() {
    let mut tardi = Tardi::default();
    let result = tardi.compile("MACRO: & ;");
    assert!(result.is_ok());

    // TODO: creates `&` macro in environment
    // TODO: executes `&` and does nothing
}
