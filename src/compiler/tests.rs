use super::*;
use crate::compiler::program::Program;
use crate::scanner::Scanner;

use pretty_assertions::assert_eq;

// TODO: more tests

fn compile(input: &str) -> Result<Program> {
    let scanner = Scanner::new(input);
    let mut compiler = Compiler::new();
    compiler.compile(scanner)
}

#[test]
fn test_compile_comparison_operators() -> Result<()> {
    let program = compile("1 2 == 3 4 != 5 6 < 7 8 > 9 10 <= 11 12 >=")?;

    let expected_ops = vec![
        "lit", "lit", "==", // 1 2 ==
        "lit", "lit", "==", "!", // 3 4 != (implemented as == !)
        "lit", "lit", "<", // 5 6 <
        "lit", "lit", ">", // 7 8 >
        "lit", "lit", ">", "!", // 9 10 <= (implemented as > !)
        "lit", "lit", "<", "!", // 11 12 >= (implemented as < !)
    ];
    let mut actual_ops = Vec::new();
    let instructions = program.get_instructions();
    let mut i = 0;
    while i < instructions.len() {
        let op = instructions[i];
        let name = program.get_op_name(op).unwrap().to_string();
        actual_ops.push(name.clone());
        if name == "lit" {
            i += 2;
        } else {
            i += 1;
        }
    }

    assert_eq!(actual_ops, expected_ops);
    Ok(())
}

#[test]
fn test_compile_return_stack_operations() -> Result<()> {
    let program = compile("42 >r r@ r>")?;

    let expected_ops = vec![
        "lit", // Push 42
        ">r",  // Move to return stack
        "r@",  // Copy from return stack
        "r>",  // Move from return stack
    ];

    let mut actual_ops = Vec::new();
    let instructions = program.get_instructions();
    let mut i = 0;
    while i < instructions.len() {
        let op = instructions[i];
        let name = program.get_op_name(op).unwrap().to_string();
        actual_ops.push(name.clone());
        if name == "lit" {
            i += 2;
        } else {
            i += 1;
        }
    }

    assert_eq!(actual_ops, expected_ops);
    Ok(())
}

#[test]
fn test_compile_word() -> Result<()> {
    let result = compile("custom_word");
    assert!(result.is_err());
    if let Err(Error::CompilerError(CompilerError::UnsupportedToken(msg))) = result {
        assert_eq!(msg, "word: custom_word");
    } else {
        panic!("Expected UnsupportedToken error");
    }
    Ok(())
}
