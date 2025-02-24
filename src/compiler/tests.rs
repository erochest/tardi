use pretty_assertions::assert_eq;

use super::*;

fn test_compile(input: &str, expected: Vec<u8>) -> Chunk {
    let result = compile(input);
    assert!(result.is_ok(), "Result: {:?}", result);
    let chunk = result.unwrap();
    assert_eq!(chunk.code, expected);
    chunk
}

fn test_compile_tokens(input: &str, expected: Vec<u8>) -> Chunk {
    let result = compile(input);
    assert!(result.is_ok(), "error: {:?}", result);
    let chunk = result.unwrap();
    assert_eq!(chunk.code, expected);
    chunk
}

#[test]
fn test_compile_simple_expression_with_div() {
    test_compile(
        "10 3 /",
        vec![0, 0, 0, 1, OpCode::Div as u8, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_simple_expression() {
    test_compile("10 3", vec![0, 0, 0, 1, OpCode::Return as u8]);
}

#[test]
fn test_compile_simple_expression_with_add() {
    test_compile(
        "10 3 +",
        vec![0, 0, 0, 1, OpCode::Add as u8, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_simple_expression_with_sub() {
    test_compile(
        "10 3 -",
        vec![0, 0, 0, 1, OpCode::Sub as u8, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_simple_expression_with_mult() {
    test_compile(
        "10 3 *",
        vec![0, 0, 0, 1, OpCode::Mult as u8, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_string() {
    test_compile(
        "\"hello\"",
        vec![OpCode::GetConstant as u8, 0, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_float() {
    test_compile(
        "3.14159",
        vec![OpCode::GetConstant as u8, 0, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_rational() {
    test_compile(
        "7/9",
        vec![OpCode::GetConstant as u8, 0, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_boolean() {
    test_compile(
        "true",
        vec![OpCode::GetConstant as u8, 0, OpCode::Return as u8],
    );
}

#[test]
fn test_compile_boolean_operators() {
    let input = "== != < <= > >= !";
    let expected = vec![
        OpCode::Equal as u8,
        OpCode::Equal as u8,
        OpCode::Not as u8,
        OpCode::Less as u8,
        OpCode::Greater as u8,
        OpCode::Not as u8,
        OpCode::Greater as u8,
        OpCode::Less as u8,
        OpCode::Not as u8,
        OpCode::Not as u8,
        OpCode::Return as u8,
    ];
    test_compile(input, expected);
}

#[test]
fn test_compile_empty_vector() {
    let input = " { } ";
    let expected = vec![0, 0, OpCode::Return as u8];
    test_compile_tokens(input, expected);
}

#[test]
fn test_compile_vector_with_items() {
    let input = "{ 0 1 1 2 3 5 8 13 }";
    let expected = vec![0, 8, OpCode::Return as u8];
    test_compile_tokens(input, expected);
}

#[test]
fn test_compile_vector_nested() {
    let input = "{
        { \"given-name\" \"Zaphod\" }
        { \"surname\" \"Beeblebrox\" }
        { \"universe\" \"Hitchhikers\" }
    }";
    let expected = vec![0, 9, OpCode::Return as u8];
    test_compile_tokens(input, expected);
}

#[test]
fn test_compile_function() {
    let input = ": double ( x -- y ) 2 * ;";
    let expected = vec![
        OpCode::Jump as u8,
        6,
        OpCode::GetConstant as u8,
        0,
        OpCode::Mult as u8,
        OpCode::Return as u8,
        OpCode::Return as u8,
    ];
    let chunk = test_compile_tokens(input, expected);
    assert!(chunk.dictionary.contains_key("double"));
}
