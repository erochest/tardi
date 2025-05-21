use std::convert::TryFrom;

use super::*;
use crate::core::Tardi;
use crate::env::Environment;
use crate::shared::unshare_clone;
use crate::value::{Pos, Value};

use crate::module::SANDBOX;
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
    ];
    let actual_ops = get_ops(environment);

    assert_eq!(actual_ops, expected_ops);

    Ok(())
}

#[test]
fn test_compile_return_stack_operations() -> Result<()> {
    let environment = compile("42 >r r@ r>")?;

    let expected_ops = vec![OpCode::Lit, OpCode::ToR, OpCode::RFetch, OpCode::RFrom];
    let actual_ops = get_ops(environment);

    assert_eq!(actual_ops, expected_ops);
    Ok(())
}

#[test]
fn test_compile_word() -> Result<()> {
    let result = compile("custom-word");
    assert!(result.is_ok());
    let env = result.unwrap().clone();
    let env = env.borrow();
    let constant = env.constants.last();
    assert!(constant.is_some());
    let constant = constant.unwrap();
    assert_eq!(constant.data, ValueData::Word("custom-word".to_string()));
    assert_eq!(constant.lexeme, Some("custom-word".to_string()));
    Ok(())
}

#[test]
fn test_compile_character_literals() -> Result<()> {
    let environment = compile("'a' '\\n' '\\t' '\\r' '\\'' '\\\\' '🦀' '\\u41' '\\u{1F600}'")?;

    let expected_ops = vec![OpCode::Lit; 9]; // One lit operation for each character

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
                (
                    0,
                    Value {
                        data: ValueData::Char('a'),
                        ..
                    },
                ) => (),
                (
                    1,
                    Value {
                        data: ValueData::Char('\n'),
                        ..
                    },
                ) => (),
                (
                    2,
                    Value {
                        data: ValueData::Char('\t'),
                        ..
                    },
                ) => (),
                (
                    3,
                    Value {
                        data: ValueData::Char('\r'),
                        ..
                    },
                ) => (),
                (
                    4,
                    Value {
                        data: ValueData::Char('\''),
                        ..
                    },
                ) => (),
                (
                    5,
                    Value {
                        data: ValueData::Char('\\'),
                        ..
                    },
                ) => (),
                (
                    6,
                    Value {
                        data: ValueData::Char('🦀'),
                        ..
                    },
                ) => (),
                (
                    7,
                    Value {
                        data: ValueData::Char('A'),
                        ..
                    },
                ) => (), // '\u41'
                (
                    8,
                    Value {
                        data: ValueData::Char('😀'),
                        ..
                    },
                ) => (), // '\u{1F600}'
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

#[test]
fn test_compile_macro_basic() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str("MACRO: & ;");

    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let ip = tardi.environment.borrow().get_op_index(SANDBOX, "&");
    assert!(ip.is_some(), "ip {:?}", ip);
    let ip = ip.unwrap();
    let lambda = tardi.environment.borrow().get_op(&0, ip);
    assert!(lambda.is_ok(), "lambda {:?}", lambda);
    let lambda = lambda.unwrap();
    assert!(lambda.borrow().immediate, "is not immediate");

    let result = tardi.execute_str("40 41 & 42");

    assert!(result.is_ok(), "ERROR MACRO use: {:?}", result);
    assert_eq!(
        tardi.stack(),
        vec![ValueData::Integer(40).into(), 41.into(), 42.into()]
    );
}

#[test]
fn test_compile_macro_scan_value() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
        use: std/scanning
        use: std/vectors
        MACRO: \
            dup >r
            scan-value lit
            r> push ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"40 42 \ +"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();
    assert_eq!(stack.len(), 3);
    assert_eq!(stack[0], 40.into());
    assert_eq!(stack[1], 42.into());
    assert_eq!(
        stack[2],
        Value {
            data: ValueData::Literal(Box::new(Value {
                data: ValueData::Word("+".to_string()),
                lexeme: Some("+".to_string()),
                pos: Some(Pos {
                    line: 1,
                    column: 9,
                    offset: 8,
                    length: 1,
                }),
            })),
            lexeme: None,
            pos: None,
        }
    );
}

#[test]
fn test_compile_macro_scan_value_list() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
            use: std/scanning
            use: std/vectors
            MACRO: [
                dup >r
                ] scan-value-list
                r> push ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"[ 40 41 42 ]"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();
    assert_eq!(stack.len(), 1);
    assert!(matches!(stack[0].data, ValueData::List(_)));
    let list = stack[0].get_list().unwrap();
    assert_eq!(3, list.len());
    assert_eq!(
        ValueData::Integer(40),
        unshare_clone(list.first().cloned().unwrap()).data,
    );
    assert_eq!(
        ValueData::Integer(41),
        unshare_clone(list.get(1).cloned().unwrap()).data,
    );
    assert_eq!(
        ValueData::Integer(42),
        unshare_clone(list.get(2).cloned().unwrap()).data,
    );
}

#[test]
fn test_compile_macro_scan_object_list_handles_flat_structures() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
            use: std/scanning
            use: std/vectors
            MACRO: [
                dup >r
                ] scan-object-list
                r> push ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"[ 40 41 42  ]"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();

    assert_eq!(stack.len(), 1);
    assert!(matches!(stack[0].data, ValueData::List(_)));

    let list = stack[0].get_list().unwrap();
    assert_eq!(3, list.len());
    assert_eq!(
        ValueData::Integer(40),
        unshare_clone(list.first().cloned().unwrap()).data
    );
    assert_eq!(
        ValueData::Integer(41),
        unshare_clone(list.get(1).cloned().unwrap()).data
    );
    assert_eq!(
        ValueData::Integer(42),
        unshare_clone(list.get(2).cloned().unwrap()).data
    );
}

#[test]
fn test_compile_macro_scan_object_list_allows_embedded_structures() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
            use: std/scanning
            use: std/vectors
            MACRO: [
                dup
                ] scan-object-list
                swap push ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"[ 40 41 42 [ 43 44 45 ] ]"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();

    assert_eq!(stack.len(), 1);
    assert!(matches!(stack[0].data, ValueData::List(_)));

    let list = stack[0].get_list().unwrap();
    assert_eq!(4, list.len());
    assert_eq!(
        ValueData::Integer(40),
        unshare_clone(list.first().cloned().unwrap()).data
    );
    assert_eq!(
        ValueData::Integer(41),
        unshare_clone(list.get(1).cloned().unwrap()).data
    );
    assert_eq!(
        ValueData::Integer(42),
        unshare_clone(list.get(2).cloned().unwrap()).data
    );

    let sublist: Value = unshare_clone(list.get(3).cloned().unwrap());
    let sublist = sublist.get_list();
    assert!(sublist.is_some());
    let sublist = sublist.unwrap();
    assert_eq!(3, sublist.len());
    assert_eq!(
        ValueData::Integer(43),
        unshare_clone(sublist.first().cloned().unwrap()).data,
    );
    assert_eq!(
        ValueData::Integer(44),
        unshare_clone(sublist.get(1).cloned().unwrap()).data,
    );
    assert_eq!(
        ValueData::Integer(45),
        unshare_clone(sublist.get(2).cloned().unwrap()).data,
    );
}

#[test]
fn test_compile_define_use_function() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
        use: std/internals
        use: std/scanning
        use: std/vectors

        MACRO: {
                dup
                } scan-object-list compile
                swap push ;

        over { >r dup r> swap } <function>
        "#,
    );
    assert!(result.is_ok(), "ERROR defining macro {{ : {:?}", result);

    // {
    //     let env = tardi.environment.borrow();
    //     let sandbox = env.get_module(SANDBOX).unwrap();
    //     log::trace!("SANDBOX");
    //     log::trace!("{:?}", sandbox);
    // }

    let result = tardi.execute_str("42 7 over");
    assert!(result.is_ok(), "ERROR executing over: {:?}", result);
    let stack = tardi.stack();

    assert_eq!(stack.len(), 3);
    assert!(matches!(stack[0].data, ValueData::Integer(42)));
    assert!(matches!(stack[1].data, ValueData::Integer(7)));
    assert!(matches!(stack[2].data, ValueData::Integer(42)));
}

// TODO: it seems like previous tests works because the outer macro call is
// building a list on the stack, just like the list that the scanner exposes for
// macros. But this will break if the outer macro is building something else,
// like a hashmap or set. Maybe I need a new test to build this out.
#[test]
fn test_compile_macro_scan_object_list_allows_heterogeneous_embedded_structures() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    // TODO: can I embed a list in a `{ ... }` lambda?
    tardi
        .execute_str(
            r#"
            use: std/internals
            use: std/scanning
            use: std/vectors

        MACRO: {
                dup
                } scan-object-list compile
                swap push ;
        "#,
        )
        .unwrap();
    tardi
        .execute_str(
            r#"
        over { >r dup r> swap } <function>
        "#,
        )
        .unwrap();

    let result = tardi.execute_str(
        r#"
            MACRO: \ scan-value over push ;
            "#,
    );
    assert!(result.is_ok(), "ERROR MACRO \\ definition: {:?}", result);
    let result = tardi.execute_str(
        r#"
            MACRO: [
                ] scan-object-list
                over push ;
            "#,
    );
    assert!(result.is_ok(), "ERROR MACRO [ definition: {:?}", result);
    let result = tardi.execute_str(
        r#"
            MACRO: :
                scan-value
                \ ; scan-object-list compile
                <function> ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO : definition: {:?}", result);

    let result = tardi.execute_str(
        r#"
        : double 2 * ;
        4 double
        "#,
    );
    assert!(result.is_ok(), "ERROR double: {:?}", result);
    let stack = tardi.stack();
    assert_eq!(stack.len(), 1, "stack = {}", ValueVec(&stack));

    let doubled = stack[0].get_integer();
    assert_eq!(Some(8), doubled);

    let result = tardi.execute_str(
        r#"
        drop
        : >name [ "name" ] swap over push ;
        "Zaphod" >name
        "#,
    );
    assert!(result.is_ok(), "ERROR >name: {:?}", result);
    let stack = tardi.stack();
    assert_eq!(stack.len(), 1, "stack = {}", ValueVec(&stack));

    let list = stack[0].get_list().unwrap();
    assert_eq!(2, list.len());
    assert_eq!(
        ValueData::String("name".to_string()),
        unshare_clone(list.first().cloned().unwrap()).data,
    );
    assert_eq!(
        ValueData::String("Zaphod".to_string()),
        unshare_clone(list.get(1).cloned().unwrap()).data,
    );
}
