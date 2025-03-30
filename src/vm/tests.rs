use super::*;
use crate::env::Environment;
use crate::error::{Error, VMError};
use crate::vm::value::{Callable, Function, Value};
use std::collections::HashMap;

#[test]
fn test_stack_operations() {
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test push and pop
    vm.push(shared(Value::Integer(42))).unwrap();
    assert_eq!(vm.stack_size(), 1);
    let value = vm.pop().unwrap();
    assert!(matches!(*value.borrow(), Value::Integer(42)));
    assert!(matches!(
        vm.pop(),
        Err(Error::VMError(VMError::StackUnderflow))
    ));

    // Test dup
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.dup().unwrap();
    assert_eq!(vm.stack_size(), 2);
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));

    // Test swap
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.swap().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(2)));

    // Test rot
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.rot().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(3)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(2)));

    // Test drop_op
    vm.push(shared(Value::Integer(42))).unwrap();
    vm.drop_op().unwrap();
    assert_eq!(vm.stack_size(), 0);
}

#[test]
fn test_basic_vm_execution() {
    let op_table = create_op_table();
    let environment = Environment::from_parameters(
        vec![Value::Integer(123)],
        vec![0, 0], // lit operation index followed by constant index
        op_table,
        HashMap::new(),
    );
    let mut vm = VM::new(shared(environment));

    vm.run().unwrap();

    // Verify the result
    let value = vm.pop().unwrap();
    assert!(matches!(*value.borrow(), Value::Integer(123)));
}

#[test]
fn test_invalid_opcode() {
    let environment = Environment::from_parameters(
        vec![],
        vec![999], // Invalid opcode
        vec![],
        HashMap::new(),
    );
    let mut vm = VM::new(shared(environment));

    assert!(matches!(
        vm.run(),
        Err(Error::VMError(VMError::InvalidOpCode(_)))
    ));
}

#[test]
fn test_function_and_lambda_operations() {
    let op_table = create_op_table();

    // Test lambda creation and execution
    let lambda_environment = Environment::from_parameters(
        vec![
            Value::Function(shared(Callable::Fn(Function {
                name: None,
                words: vec!["2".to_string(), "3".to_string(), "*".to_string()],
                instructions: 5, // Index where the lambda instructions start
            }))),
            Value::Integer(2),
            Value::Integer(3),
        ],
        vec![
            0,
            0,                          // lit (push lambda)
            OpCode::CallStack as usize, // call the lambda
            OpCode::Jump as usize,
            11,
            0,
            1,
            0,
            2,
            OpCode::Multiply as usize,
            OpCode::Return as usize,
        ],
        op_table.clone(),
        HashMap::new(),
    );

    let mut vm = VM::new(shared(lambda_environment));
    vm.run().unwrap();

    // Verify the result of lambda execution (2 * 3 = 6)
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(6)));

    // Test function definition and execution
    let function_environment = Environment::from_parameters(
        vec![
            Value::String("triple".to_string()),
            Value::Function(shared(Callable::Fn(Function {
                name: None,
                words: vec!["3".to_string(), "*".to_string()],
                instructions: 7, // Index where the function instructions start
            }))),
            Value::Integer(3),
            Value::Integer(4),
        ],
        vec![
            0,
            0,
            0,
            1,
            OpCode::Function as usize, // define the function
            OpCode::Jump as usize,
            11,
            0,
            2,
            OpCode::Multiply as usize,
            OpCode::Return as usize,
            0,
            3,
            OpCode::Call as usize,
            7,
        ],
        op_table,
        HashMap::new(),
    );

    let mut vm = VM::new(shared(function_environment));
    vm.run().unwrap();

    // Verify the result of function execution (4 * 3 = 12)
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(12)));
}

#[test]
fn test_return_stack_operations() {
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test >r (to_r)
    vm.push(shared(Value::Integer(42))).unwrap();
    vm.to_r().unwrap();
    assert_eq!(vm.stack_size(), 0);

    // Test r> (r_from)
    vm.r_from().unwrap();
    assert_eq!(vm.stack_size(), 1);
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(42)));

    // Test r@ (r_fetch)
    vm.push(shared(Value::Integer(10))).unwrap();
    vm.to_r().unwrap();
    vm.r_fetch().unwrap();
    assert_eq!(vm.stack_size(), 1);
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(10)));
    vm.r_from().unwrap(); // Clear the return stack

    // Test return stack overflow
    for i in 0..1024 {
        vm.push(shared(Value::Integer(1))).unwrap();
        match vm.to_r() {
            Ok(_) => continue,
            Err(Error::VMError(VMError::ReturnStackOverflow)) => {
                // We've hit the overflow, verify we pushed the expected number of values
                assert!(i > 0);
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    // Clear the return stack (only pop what we successfully pushed)
    while vm.r_from().is_ok() {}

    // Test return stack underflow
    assert!(matches!(
        vm.r_from(),
        Err(Error::VMError(VMError::ReturnStackUnderflow))
    ));
    assert!(matches!(
        vm.r_fetch(),
        Err(Error::VMError(VMError::ReturnStackUnderflow))
    ));
}

#[test]
fn test_arithmetic_operations() {
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test integer addition
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.push(shared(Value::Integer(4))).unwrap();
    vm.add().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(7)));

    // Test float addition
    vm.push(shared(Value::Float(3.5))).unwrap();
    vm.push(shared(Value::Float(1.5))).unwrap();
    vm.add().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Float(5.0)));

    // Test mixed addition (integer + float)
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.push(shared(Value::Float(1.5))).unwrap();
    vm.add().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Float(3.5)));

    // Test subtraction
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.subtract().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(2)));

    // Test multiplication
    vm.push(shared(Value::Integer(4))).unwrap();
    vm.push(shared(Value::Integer(3))).unwrap();
    vm.multiply().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(12)));

    // Test division
    vm.push(shared(Value::Integer(10))).unwrap();
    vm.push(shared(Value::Integer(2))).unwrap();
    vm.divide().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(5)));
}

#[test]
fn test_arithmetic_errors() {
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test division by zero (integer)
    vm.push(shared(Value::Integer(10))).unwrap();
    vm.push(shared(Value::Integer(0))).unwrap();
    assert!(matches!(
        vm.divide(),
        Err(Error::VMError(VMError::DivisionByZero))
    ));

    // Test division by zero (float)
    vm.push(shared(Value::Float(10.0))).unwrap();
    vm.push(shared(Value::Float(0.0))).unwrap();
    assert!(matches!(
        vm.divide(),
        Err(Error::VMError(VMError::DivisionByZero))
    ));

    // Test type mismatch
    vm.push(shared(Value::Integer(1))).unwrap();
    vm.push(shared(Value::Boolean(true))).unwrap();
    assert!(matches!(
        vm.add(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test stack underflow
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));
    assert!(matches!(
        vm.add(),
        Err(Error::VMError(VMError::StackUnderflow))
    ));
}

#[test]
fn test_character_operations() {
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test basic character handling
    vm.push(shared(Value::Char('a'))).unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Char('a')));

    // Test character literals in environment execution
    let op_table = create_op_table();
    let environment = Environment::from_parameters(
        vec![Value::Char('a'), Value::Char('\n'), Value::Char('🦀')],
        vec![
            0, 0, // lit 'a'
            0, 1, // lit '\n'
            0, 2, // lit '🦀'
        ],
        op_table,
        HashMap::new(),
    );

    let mut vm = VM::new(shared(environment));
    vm.run().unwrap();

    // Verify the characters were pushed onto the stack in the correct order
    let top = vm.pop().unwrap();
    let top = top.borrow();
    assert!(matches!(*top, Value::Char('🦀')), "stack top: {:?}", top);
    let top = vm.pop().unwrap();
    let top = top.borrow();
    assert!(matches!(*top, Value::Char('\n')), "stack top: {:?}", top);
    let top = vm.pop().unwrap();
    let top = top.borrow();
    assert!(matches!(*top, Value::Char('a')), "stack top: {:?}", top);

    // Test stack operations with characters
    vm.push(shared(Value::Char('x'))).unwrap();
    vm.push(shared(Value::Char('y'))).unwrap();
    vm.dup().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Char('y')));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Char('y')));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Char('x')));

    // Test comparison operations with characters
    vm.push(shared(Value::Char('a'))).unwrap();
    vm.push(shared(Value::Char('a'))).unwrap();
    vm.equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    vm.push(shared(Value::Char('a'))).unwrap();
    vm.push(shared(Value::Char('b'))).unwrap();
    vm.less().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test type mismatch with characters
    vm.push(shared(Value::Char('a'))).unwrap();
    vm.push(shared(Value::Integer(1))).unwrap();
    assert!(matches!(
        vm.equal(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));
}

#[test]
fn test_comparison_operations() {
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test equal
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(false)));

    // Test less than
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.less().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test greater than
    vm.push(shared(Value::Integer(6))).unwrap();
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.greater().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test comparison with different types
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Float(5.0))).unwrap();
    vm.equal().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test comparison error
    vm.push(shared(Value::Integer(5))).unwrap();
    vm.push(shared(Value::Boolean(true))).unwrap();
    assert!(matches!(
        vm.equal(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test NOT operation
    vm.push(shared(Value::Boolean(true))).unwrap();
    vm.not().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(false)));

    vm.push(shared(Value::Boolean(false))).unwrap();
    vm.not().unwrap();
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Boolean(true)));

    // Test NOT operation error
    vm.push(shared(Value::Integer(5))).unwrap();
    assert!(matches!(
        vm.not(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));
}

#[test]
fn test_function_and_lambda_errors() {
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test calling a non-function value
    vm.push(shared(Value::Integer(42))).unwrap();
    assert!(matches!(
        vm.call_stack(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test function definition with invalid name
    vm.push(shared(Value::Integer(42))).unwrap(); // Invalid name (not a string)
    vm.push(shared(Value::Function(shared(Callable::Fn(Function {
        name: None,
        words: vec!["2".to_string(), "*".to_string()],
        instructions: 0,
    })))))
    .unwrap();
    assert!(matches!(
        vm.function(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test function definition with invalid lambda
    vm.push(shared(Value::String("test".to_string()))).unwrap();
    vm.push(shared(Value::Integer(42))).unwrap(); // Invalid lambda
    assert!(matches!(
        vm.function(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test calling without a environment loaded
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test return operation with empty return stack
    assert!(matches!(vm.return_op(), Err(Error::VMError(VMError::Exit))));

    // Test return operation with invalid return address
    vm.push_return(shared(Value::Integer(42))).unwrap(); // Not an address
    assert!(matches!(
        vm.return_op(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));
}

#[test]
fn test_jump_operations() {
    // Test basic jump
    let op_table = create_op_table();
    let jump_environment = Environment::from_parameters(
        vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        vec![
            0,
            0, // lit 1
            OpCode::Jump as usize,
            6, // jump to position 5
            0,
            1, // lit 2 (skipped)
            0,
            2, // lit 3
            OpCode::Return as usize,
        ],
        op_table.clone(),
        HashMap::new(),
    );
    let mut vm = VM::new(shared(jump_environment));

    vm.run().unwrap();

    // Should have pushed 1 and 3, skipping 2
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(3)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));

    // Test jump_stack
    let jump_stack_environment = Environment::from_parameters(
        vec![
            Value::Integer(1),
            Value::Address(7),
            Value::Integer(2),
            Value::Integer(3),
        ],
        vec![
            0,
            0, // lit 1
            0,
            1, // lit address(7)
            OpCode::JumpStack as usize,
            0,
            2, // lit 2 (skipped)
            0,
            3, // lit 3
        ],
        op_table,
        HashMap::new(),
    );

    let mut vm = VM::new(shared(jump_stack_environment));
    vm.run().unwrap();

    // Should have pushed 1 and 3, skipping 2
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(3)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));

    // Test jump errors
    let environment = Environment::default();
    let mut vm = VM::new(shared(environment));

    // Test jump_stack with invalid address type
    vm.push(shared(Value::Integer(42))).unwrap(); // Not an address
    assert!(matches!(
        vm.jump_stack(),
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));
}
