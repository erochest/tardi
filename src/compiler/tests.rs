use std::convert::TryFrom;

use super::*;
use crate::core::Tardi;
use crate::env::Environment;
use crate::shared::unshare_clone;
use crate::value::Value;

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
    let result = compile("custom-word");
    assert!(result.is_ok());
    let env = result.unwrap().clone();
    assert_eq!(
        env.borrow().constants.last(),
        // TODO: need to make these values reflect the actual token,
        // not the half-assed values we're stubbing in
        Some(&Value::Token(Token {
            token_type: TokenType::Word("custom-word".to_string()),
            // line: 1, column: 1, offset: 0,
            line: 0,
            column: 0,
            offset: 0,
            length: 11,
            lexeme: "custom-word".to_string(),
        }))
    );
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

#[test]
fn test_compile_macro_basic() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str("MACRO: & ;");

    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);
    assert!(tardi
        .environment
        .borrow()
        .is_macro_trigger(&TokenType::Word("&".to_string())));

    let result = tardi.execute_str("40 41 & 42");

    assert!(result.is_ok(), "ERROR MACRO use: {:?}", result);
    assert_eq!(
        tardi.stack(),
        vec![Value::Integer(40), 41.into(), 42.into()]
    );
}

#[test]
fn test_compile_macro_scan_token() {
    // env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
        MACRO: \
            dup >r
            scan-token lit
            r> append ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"40 42 \ +"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();
    assert_eq!(stack.len(), 3);
    assert_eq!(stack[0], 40.into());
    assert_eq!(stack[1], 42.into());
    eprintln!("{:#?}", stack[2]);
    assert_eq!(
        stack[2],
        Value::Token(Token::new(TokenType::Plus, 1, 9, 8, 1, "+".to_string()))
    );
}

#[test]
fn test_compile_macro_scan_token_list() {
    env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
            MACRO: [
                dup >r
                ] scan-token-list
                r> append ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"[ 40 41 42 ]"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();
    assert_eq!(stack.len(), 1);
    assert!(matches!(stack[0], Value::List(_)));
    let list = stack[0].get_list().unwrap();
    assert_eq!(3, list.len());
    assert_eq!(
        Value::Integer(40),
        unshare_clone(list.get(0).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(41),
        unshare_clone(list.get(1).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(42),
        unshare_clone(list.get(2).cloned().unwrap())
    );
}

#[test]
fn test_compile_macro_scan_value_list_handles_flat_structures() {
    env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
            MACRO: [
                dup >r
                ] scan-value-list
                r> append ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"[ 40 41 42  ]"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();

    assert_eq!(stack.len(), 1);
    assert!(matches!(stack[0], Value::List(_)));

    // TODO: these are getting wrapped in Token's. i'm probably
    // not thinking clearly about code-as-data and when it should
    // be code and when it should be data. Maybe I need to
    // revisit the differences between `Taken`, `TokenType`,
    // and `Value`.
    let list = stack[0].get_list().unwrap();
    assert_eq!(3, list.len());
    assert_eq!(
        Value::Integer(40),
        unshare_clone(list.get(0).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(41),
        unshare_clone(list.get(1).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(42),
        unshare_clone(list.get(2).cloned().unwrap())
    );
}

#[test]
fn test_compile_macro_scan_value_list_allows_embedded_structures() {
    env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
            "over" { >r dup >r swap } <function>
            MACRO: [
                ] scan-value-list
                over append ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(r#"[ 40 41 42 [ 43 44 45 ] ]"#);
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();

    assert_eq!(stack.len(), 1);
    assert!(matches!(stack[0], Value::List(_)));

    let list = stack[0].get_list().unwrap();
    assert_eq!(4, list.len());
    assert_eq!(
        Value::Integer(40),
        unshare_clone(list.get(0).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(41),
        unshare_clone(list.get(1).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(42),
        unshare_clone(list.get(2).cloned().unwrap())
    );

    let sublist: Value = unshare_clone(list.get(3).cloned().unwrap());
    let sublist = sublist.get_list();
    assert!(sublist.is_some());
    let sublist = sublist.unwrap();
    assert_eq!(3, sublist.len());
    assert_eq!(
        Value::Integer(43),
        unshare_clone(sublist.get(0).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(44),
        unshare_clone(sublist.get(1).cloned().unwrap())
    );
    assert_eq!(
        Value::Integer(45),
        unshare_clone(sublist.get(2).cloned().unwrap())
    );
}

// TODO: it seems like previous tests works because the outer macro call is
// building a list on the stack, just like the list that the scanner exposes for
// macros. But this will break if the outer macro is building something else,
// like a hashmap or set. Maybe I need a new test to build this out.
#[test]
fn test_compile_macro_scan_value_list_allows_heterogeneous_embedded_structures() {
    env_logger::init();
    let mut tardi = Tardi::default();

    let result = tardi.execute_str(
        r#"
            "over" { >r dup >r swap } <function>
            MACRO: [
                ] scan-value-list
                over append ;
            MACRO: :
                scan-token
                ; scan-value-list
                <function>
                over append ;
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO definition: {:?}", result);

    let result = tardi.execute_str(
        r#"
        : double * 2 ;
        4 double
        : >name [ "name" ] append ;
        "Zaphod" >name
        "#,
    );
    assert!(result.is_ok(), "ERROR MACRO execution: {:?}", result);
    let stack = tardi.stack();

    assert_eq!(stack.len(), 2);
    assert!(matches!(stack[0], Value::List(_)));

    let doubled = stack[0].get_integer();
    assert_eq!(Some(8), doubled);

    let list = stack[1].get_list().unwrap();
    assert_eq!(2, list.len());
    assert_eq!(
        Value::String(">name".to_string()),
        unshare_clone(list.get(0).cloned().unwrap())
    );
    assert_eq!(
        Value::String("Zaphod".to_string()),
        unshare_clone(list.get(1).cloned().unwrap())
    );
}
