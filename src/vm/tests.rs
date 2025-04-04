use super::*;
use crate::env::Environment;
use crate::error::{Error, VMError};
use crate::vm::value::{Callable, Function, Value};
use crate::Tardi;
use std::collections::HashMap;
use std::fmt::Debug;

fn eval(input: &str) -> Result<Vec<Value>> {
    let mut tardi = Tardi::default();
    let result = tardi.execute_str(input);
    result.map(|_| tardi.executor.stack().clone())
}

fn assert_is_ok<T: Debug>(input: &str, result: &Result<T>) {
    assert!(
        result.is_ok(),
        "result input: {:?} / error: {:?}",
        input,
        result
    );
}

#[test]
fn test_stack_operations() {
    // Test push and pop
    let mut stack = eval("42").unwrap();
    assert_eq!(stack.len(), 1);
    let value = stack.pop().unwrap();
    assert!(matches!(value, Value::Integer(42)));

    // Test dup
    let mut stack = eval("1 dup").unwrap();
    assert_eq!(stack.len(), 2);
    assert!(matches!(stack.pop().unwrap(), Value::Integer(1)));
    assert!(matches!(stack.pop().unwrap(), Value::Integer(1)));

    // Test swap
    let mut stack = eval("1 2 swap").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Integer(1)));
    assert!(matches!(stack.pop().unwrap(), Value::Integer(2)));

    // Test rot
    let mut stack = eval("1 2 3 rot").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Integer(1)));
    assert!(matches!(stack.pop().unwrap(), Value::Integer(3)));
    assert!(matches!(stack.pop().unwrap(), Value::Integer(2)));

    // Test drop_op
    let mut stack = eval("42 drop").unwrap();
    assert_eq!(stack.len(), 0);

    let mut stack = eval("10 11 12 13 stack-size").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Integer(4)));
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
    let mut vm = VM::new();

    vm.run(shared(environment)).unwrap();

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
    let mut vm = VM::new();

    assert!(matches!(
        vm.run(shared(environment)),
        Err(Error::VMError(VMError::InvalidOpCode(_, _)))
    ));
}

#[test]
fn test_function_and_lambda_operations() {
    let op_table = create_op_table();

    // Test lambda creation and execution
    let lambda_environment = Environment::from_parameters(
        vec![
            Value::Function(Callable::Fn(Function {
                name: None,
                words: vec!["2".to_string(), "3".to_string(), "*".to_string()],
                ip: 5, // Index where the lambda instructions start
            })),
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

    let mut vm = VM::new();
    vm.run(shared(lambda_environment)).unwrap();

    // Verify the result of lambda execution (2 * 3 = 6)
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(6)));

    // Test function definition and execution
    let function_environment = Environment::from_parameters(
        vec![
            Value::String("triple".to_string()),
            Value::Function(Callable::Fn(Function {
                name: None,
                words: vec!["3".to_string(), "*".to_string()],
                ip: 7, // Index where the function instructions start
            })),
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

    let mut vm = VM::new();
    vm.run(shared(function_environment)).unwrap();

    // Verify the result of function execution (4 * 3 = 12)
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(12)));
}

#[test]
fn test_return_stack_operations() {
    // Test >r (to_r)
    let result = eval("42 >r stack-size r> drop");
    assert_is_ok("42 >r stack-size r> drop", &result);
    let mut stack = result.unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Integer(0)));

    // Test r> (r_from)
    let mut stack = eval("42 >r 7 r>").unwrap();
    assert_eq!(stack.len(), 2);
    assert!(matches!(stack.pop().unwrap(), Value::Integer(42)));
    assert!(matches!(stack.pop().unwrap(), Value::Integer(7)));

    // Test r@ (r_fetch)
    let mut stack = eval("10 >r r@ r>").unwrap();
    assert_eq!(stack.len(), 2);
    assert!(matches!(stack.pop().unwrap(), Value::Integer(10)));
    assert!(matches!(stack.pop().unwrap(), Value::Integer(10)));

    // Test return stack overflow
    let script = (0..2048)
        .map(|n| format!("{} >r", n))
        .collect::<Vec<_>>()
        .join(" ");
    let result = eval(&script);
    assert!(
        matches!(result, Err(Error::VMError(VMError::ReturnStackOverflow)),),
        "actual result {:?}",
        result
    );

    // Test return stack underflow
    let result = eval("r>");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::ReturnStackUnderflow)),
    ));
    let result = eval("r@");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::ReturnStackUnderflow)),
    ));
}

#[test]
fn test_arithmetic_operations() {
    // Test integer addition
    let mut stack = eval("3 4 +").unwrap();
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Integer(7)), "{:?} != {:?}", top, 7);

    // Test float addition
    let mut stack = eval("3.5 1.5 +").unwrap();
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Float(5.0)), "{:?} != {:?}", top, 5.0);

    // Test mixed addition (integer + float)
    let mut stack = eval("2 1.5 +").unwrap();
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Float(3.5)), "{:?} != {:?}", top, 3.5);

    // Test subtraction
    let mut stack = eval("5 3 -").unwrap();
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Integer(2)), "{:?} != {:?}", top, 2);

    // Test multiplication
    let mut stack = eval("4 3 *").unwrap();
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Integer(12)), "{:?} != {:?}", top, 12);

    // Test division
    let mut stack = eval("10 2 /").unwrap();
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Integer(5)), "{:?} != {:?}", top, 5);
}

#[test]
fn test_arithmetic_errors() {
    // Test division by zero (integer)
    let result = eval("10 0 /");
    assert!(
        matches!(result, Err(Error::VMError(VMError::DivisionByZero))),
        "original result: {:?}",
        result
    );

    // Test division by zero (float)
    let result = eval("10.0 0.0 /");
    assert!(
        matches!(result, Err(Error::VMError(VMError::DivisionByZero))),
        "original result: {:?}",
        result
    );

    // Test type mismatch
    let result = eval("1 #t +");
    assert!(
        matches!(result, Err(Error::VMError(VMError::TypeMismatch(_)))),
        "original result: {:?}",
        result
    );

    // Test stack underflow
    let result = eval("+");
    assert!(
        matches!(result, Err(Error::VMError(VMError::StackUnderflow))),
        "original result: {:?}",
        result
    );
}

#[test]
fn test_character_operations() {
    // Test basic character handling
    let mut stack = eval("'a'").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Char('a')));

    // Test character literals in environment execution
    let mut stack = eval("'a' '\n' '🦀'").unwrap();

    // Verify the characters were pushed onto the stack in the correct order
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Char('🦀')), "stack top: {:?}", top);
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Char('\n')), "stack top: {:?}", top);
    let top = stack.pop().unwrap();
    assert!(matches!(top, Value::Char('a')), "stack top: {:?}", top);

    // Test stack operations with characters
    let mut stack = eval("'x' 'y' dup").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Char('y')));
    assert!(matches!(stack.pop().unwrap(), Value::Char('y')));
    assert!(matches!(stack.pop().unwrap(), Value::Char('x')));

    // Test comparison operations with characters
    let mut stack = eval("'a' 'a' ==").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(true)));

    let mut stack = eval("'a' 'b' <").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(true)));

    // Test type mismatch with characters
    let result = eval("'a' 1 ==");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));
}

#[test]
fn test_comparison_operations() {
    // Test equal
    let mut stack = eval("5 5 ==").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(true)));

    let mut stack = eval("5 6 ==").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(false)));

    // Test less than
    let mut stack = eval("5 6 <").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(true)));

    // Test greater than
    let mut stack = eval("6 5 >").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(true)));

    // Test comparison with different types
    let mut stack = eval("5 5.0 ==").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(true)));

    // Test comparison error
    let result = eval("5 #t ==");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test NOT operation
    let mut stack = eval("#t !").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(false)));

    let mut stack = eval("#f !").unwrap();
    assert!(matches!(stack.pop().unwrap(), Value::Boolean(true)));
}

#[test]
fn test_function_and_lambda_errors() {
    // Test calling a non-function value
    let result = eval("42 call");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test function definition with invalid name
    let result = eval("42 { 2 * } <function>");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test function definition with invalid lambda
    let result = eval("\"test\" 42 <function>");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test return operation with invalid return address
    let result = eval("{ 42 >r } call");
    assert!(matches!(
        result,
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
    let jump_environment = shared(jump_environment);
    let mut vm = VM::new();

    vm.run(jump_environment.clone()).unwrap();

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

    let jump_stack_environment = shared(jump_stack_environment);
    let mut vm = VM::new();
    vm.run(jump_stack_environment.clone()).unwrap();

    // Should have pushed 1 and 3, skipping 2
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(3)));
    assert!(matches!(*vm.pop().unwrap().borrow(), Value::Integer(1)));

    // TODO: uncomment once there's a word for <jump-stack>
    // // Test jump errors
    // let environment = shared(Environment::with_builtins());
    // let mut vm = VM::new();

    // // Test jump_stack with invalid address type
    // let result = eval(environment.clone(), &mut vm, "42");
    // vm.push(shared(Value::Integer(42))).unwrap(); // Not an address
    // assert!(matches!(
    //     vm.jump_stack(),
    //     Err(Error::VMError(VMError::TypeMismatch(_)))
    // ));
}
