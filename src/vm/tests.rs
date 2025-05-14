use super::*;
use crate::error::{Error, VMError};
use crate::value::Value;
use crate::Tardi;
use std::fmt::Debug;

use pretty_assertions::{assert_eq, assert_ne};

fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

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
    assert!(matches!(
        value,
        Value {
            data: ValueData::Integer(42),
            ..
        }
    ));

    // Test dup
    let mut stack = eval("1 dup").unwrap();
    assert_eq!(stack.len(), 2);
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(1),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(1),
            ..
        }
    ));

    // Test swap
    let mut stack = eval("1 2 swap").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(1),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(2),
            ..
        }
    ));

    // Test rot
    let mut stack = eval("1 2 3 rot").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(1),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(3),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(2),
            ..
        }
    ));

    // Test drop_op
    let stack = eval("42 drop").unwrap();
    assert_eq!(stack.len(), 0);

    let mut stack = eval("10 11 12 13 stack-size").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(4),
            ..
        }
    ));
}

#[test]
fn test_basic_vm_execution() {
    let mut tardi = Tardi::default();
    let environment = tardi.environment.clone();
    environment.borrow_mut().constants = vec![ValueData::Integer(123).into()];
    environment.borrow_mut().instructions = vec![0, 0];

    let result = tardi.execute_ip(0);
    assert!(result.is_ok(), "basic-vm-execution error: {:?}", result);

    // Verify the result
    let value = tardi.executor.stack.pop().unwrap();
    assert!(matches!(
        *value.borrow(),
        Value {
            data: ValueData::Integer(123),
            ..
        }
    ));
}

#[test]
fn test_invalid_opcode() {
    let mut tardi = Tardi::default();
    let env = tardi.environment.clone();
    env.borrow_mut().instructions = vec![999];

    let result = tardi.execute_ip(0);

    assert!(matches!(
        result,
        Err(Error::VMError(VMError::InvalidOpCode(_, _)))
    ));
}

#[test]
fn test_function_and_lambda_operations() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    tardi
        .execute_str(
            r#"
        MACRO: {
                dup
                } scan-object-list compile
                swap append ;
        "#,
        )
        .unwrap();
    let result = tardi.execute_str("{ 2 3 * } apply");
    assert!(result.is_ok());

    // Verify the result of lambda execution (2 * 3 = 6)
    assert!(matches!(
        *tardi.executor.stack.pop().unwrap().borrow(),
        Value {
            data: ValueData::Integer(6),
            ..
        },
    ));

    // Function defined with `<function>` have to be defined in a previous input string.
    // Macro-defined functions can be used in the same input string.
    let result = tardi.execute_str("triple { 3 * } <function>");
    assert!(result.is_ok());

    let result = tardi.execute_str(
        r#"
        4 triple
        "#,
    );
    assert!(result.is_ok());

    // Verify the result of function execution (4 * 3 = 12)
    assert!(
        matches!(
            *tardi.executor.stack.pop().unwrap().borrow(),
            Value {
                data: ValueData::Integer(12),
                ..
            }
        ),
        "stack = {}",
        tardi
            .executor
            .stack
            .iter()
            .map(|v| format!("{}", v.borrow()))
            .collect::<Vec<String>>()
            .join(" "),
    );
}

#[test]
fn test_return_stack_operations() {
    // Test >r (to_r)
    let result = eval("42 >r stack-size r> drop");
    assert_is_ok("42 >r stack-size r> drop", &result);
    let mut stack = result.unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(0),
            ..
        }
    ));

    // Test r> (r_from)
    let mut stack = eval("42 >r 7 r>").unwrap();
    assert_eq!(stack.len(), 2);
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(42),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(7),
            ..
        }
    ));

    // Test r@ (r_fetch)
    let mut stack = eval("10 >r r@ r>").unwrap();
    assert_eq!(stack.len(), 2);
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(10),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Integer(10),
            ..
        }
    ));

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
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Integer(7),
                ..
            }
        ),
        "{:?} != {:?}",
        top,
        7
    );

    // Test float addition
    let mut stack = eval("3.5 1.5 +").unwrap();
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Float(5.0),
                ..
            }
        ),
        "{:?} != {:?}",
        top,
        5.0
    );

    // Test mixed addition (integer + float)
    let mut stack = eval("2 1.5 +").unwrap();
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Float(3.5),
                ..
            }
        ),
        "{:?} != {:?}",
        top,
        3.5
    );

    // Test subtraction
    let mut stack = eval("5 3 -").unwrap();
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Integer(2),
                ..
            }
        ),
        "{:?} != {:?}",
        top,
        2
    );

    // Test multiplication
    let mut stack = eval("4 3 *").unwrap();
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Integer(12),
                ..
            }
        ),
        "{:?} != {:?}",
        top,
        12
    );

    // Test division
    let mut stack = eval("10 2 /").unwrap();
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Integer(5),
                ..
            }
        ),
        "{:?} != {:?}",
        top,
        5
    );
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
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Char('a'),
            ..
        }
    ));

    // Test character literals in environment execution
    let mut stack = eval("'a' '\n' '🦀'").unwrap();

    // Verify the characters were pushed onto the stack in the correct order
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Char('🦀'),
                ..
            }
        ),
        "stack top: {:?}",
        top
    );
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Char('\n'),
                ..
            }
        ),
        "stack top: {:?}",
        top
    );
    let top = stack.pop().unwrap();
    assert!(
        matches!(
            top,
            Value {
                data: ValueData::Char('a'),
                ..
            }
        ),
        "stack top: {:?}",
        top
    );

    // Test stack operations with characters
    let mut stack = eval("'x' 'y' dup").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Char('y'),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Char('y'),
            ..
        }
    ));
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Char('x'),
            ..
        }
    ));

    // Test comparison operations with characters
    let mut stack = eval("'a' 'a' ==").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(true),
            ..
        }
    ));

    let mut stack = eval("'a' 'b' <").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(true),
            ..
        }
    ));

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
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(true),
            ..
        }
    ));

    let mut stack = eval("5 6 ==").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(false),
            ..
        }
    ));

    // Test less than
    let mut stack = eval("5 6 <").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(true),
            ..
        }
    ));

    // Test greater than
    let mut stack = eval("6 5 >").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(true),
            ..
        }
    ));

    // Test comparison with different types
    let mut stack = eval("5 5.0 ==").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(true),
            ..
        }
    ));

    // Test comparison error
    let result = eval("5 #t ==");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));

    // Test NOT operation
    let mut stack = eval("#t !").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(false),
            ..
        }
    ));

    let mut stack = eval("#f !").unwrap();
    assert!(matches!(
        stack.pop().unwrap(),
        Value {
            data: ValueData::Boolean(true),
            ..
        }
    ));
}

#[test]
fn test_function_and_lambda_errors() {
    // Test calling a non-function value
    let result = eval("42 apply");
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
    let result = eval("{ 42 >r } apply");
    assert!(matches!(
        result,
        Err(Error::VMError(VMError::TypeMismatch(_)))
    ));
}

#[test]
fn test_jump_operations() {
    let mut tardi = Tardi::default();
    let env = tardi.environment.clone();

    // Test basic jump
    env.borrow_mut().constants = vec![
        ValueData::Integer(1).into(),
        ValueData::Integer(2).into(),
        ValueData::Integer(3).into(),
    ];
    env.borrow_mut().instructions = vec![
        0,
        0, // lit 1
        OpCode::Jump as usize,
        6, // jump to position 5
        0,
        1, // lit 2 (skipped)
        0,
        2, // lit 3
        OpCode::Return as usize,
    ];

    tardi.execute_ip(0).unwrap();

    // Should have pushed 1 and 3, skipping 2
    assert!(matches!(
        *tardi.executor.stack.pop().unwrap().borrow(),
        Value {
            data: ValueData::Integer(3),
            ..
        }
    ));
    assert!(matches!(
        *tardi.executor.stack.pop().unwrap().borrow(),
        Value {
            data: ValueData::Integer(1),
            ..
        }
    ));

    // Test jump_stack
    env.borrow_mut().constants = vec![
        ValueData::Integer(1).into(),
        ValueData::Address(7).into(),
        ValueData::Integer(2).into(),
        ValueData::Integer(3).into(),
    ];
    env.borrow_mut().instructions = vec![
        0,
        0, // lit 1
        0,
        1, // lit address(7)
        OpCode::JumpStack as usize,
        0,
        2, // lit 2 (skipped)
        0,
        3, // lit 3
    ];

    tardi.execute_ip(0).unwrap();

    // Should have pushed 1 and 3, skipping 2
    assert!(matches!(
        *tardi.executor.stack.pop().unwrap().borrow(),
        Value {
            data: ValueData::Integer(3),
            ..
        }
    ));
    assert!(matches!(
        *tardi.executor.stack.pop().unwrap().borrow(),
        Value {
            data: ValueData::Integer(1),
            ..
        }
    ));

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

#[test]
fn test_clear() {
    let input = "1 2 3 4 5 clear";
    let mut tardi = Tardi::new(None).unwrap();

    let result = tardi.execute_str(input);

    assert_is_ok(input, &result);
    let stack = tardi.executor.stack;
    assert!(
        stack.is_empty(),
        "Expected stack to be empty, got {:?}",
        stack
    );
}

#[test]
fn test_predeclare_function_adds_undefined_function_to_op_table() {
    let word = "even?".to_string();
    let input = r#" even? <predeclare-function> "#;
    let mut tardi = Tardi::new(None).unwrap();
    let env = tardi.environment.clone();
    let next_index = (*env).borrow().op_table.len();

    let result = tardi.execute_str(input);

    assert_is_ok(input, &result);
    let env = tardi.environment.clone();
    let index = tardi.compiler.current_module().and_then(|m| m.get(&word));
    assert!(index.is_some(), "export {} = {:?}", word, index);
    let index = index.unwrap();
    assert_eq!(index, next_index);
    let lambda = (*env).borrow().op_table[next_index].clone();
    assert_eq!((*lambda).borrow().name, Some(word.clone()));
    assert_eq!((*lambda).borrow().defined, false);
    assert_eq!((*lambda).borrow().get_ip(), Some(0));
}

#[test]
fn test_function_defines_predeclared_function() {
    init_logging();
    // TODO: does predeclaring _have_ to happen in pass1?
    let setup = r#"
        MACRO: \ dup scan-value swap append ;
        MACRO: :
                scan-value
                dup <predeclare-function>
                \ ; scan-object-list compile
                <function> ;
        MACRO: [
                dup
                ] scan-object-list compile
                swap append ;
        "#;
    let word = "even?".to_string();
    let input = r#"
        : even?   dup 0 == [ drop #t ] [ 1 - even? ! ] ? apply ;
        "#;
    // let mut tardi = Tardi::with_bootstrap(None).unwrap();
    let mut tardi = Tardi::default();
    let next_index = (*tardi.environment).borrow().op_table.len();

    let result = tardi.execute_str(setup);
    assert_is_ok(setup, &result);
    let result = tardi.execute_str(input);

    assert_is_ok(input, &result);
    let env = tardi.environment.clone();
    let index = env.borrow().get_op_index("std/sandbox", "even?");
    assert!(index.is_some(), "export {} = {:?}", word, index);
    let index = index.unwrap();
    assert_eq!(index, next_index);
    let lambda = (*env).borrow().op_table[next_index].clone();
    assert_eq!((*lambda).borrow().name, Some(word.clone()));
    assert_eq!((*lambda).borrow().defined, true);
    assert_ne!((*lambda).borrow().get_ip(), Some(0));
}

#[test]
fn test_call_wont_execute_predeclared_function() {
    init_logging();

    let setup = r#" even? <predeclare-function> "#;
    let input = r#" 7 even? "#;
    let mut tardi = Tardi::new(None).unwrap();

    let result = tardi.execute_str(setup);
    assert_is_ok(setup, &result);
    let result = tardi.execute_str(input);

    assert!(result.is_err());
}

#[test]
fn test_call_will_execute_defined_predeclared_function() {
    init_logging();
    // TODO: does predeclaring _have_ to happen in pass1?
    let setup = r#"
        MACRO: \ dup scan-value swap append ;
        MACRO: :
                scan-value
                dup <predeclare-function>
                \ ; scan-object-list compile
                <function> ;
        MACRO: [
                dup
                ] scan-object-list compile
                swap append ;
        "#;
    let input = r#"
        : even?   dup 0 == [ drop #t ] [ 1 - even? ! ] ? apply ;
        1 even?
        2 even?
        "#;
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(setup);
    assert_is_ok(setup, &result);
    let result = tardi.execute_str(input);

    assert_is_ok(input, &result);
    let stack = tardi
        .stack()
        .into_iter()
        .map(|v| v.get_boolean())
        .collect::<Vec<_>>();
    assert_eq!(stack, vec![Some(false), Some(true)]);
}
