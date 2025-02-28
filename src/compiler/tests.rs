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

#[test]
fn test_compile_function_application() {
    // env_logger::builder().init();
    let input = "
        : double ( x -- y ) 2 * ;
        4 double
        5 double
        ";
    let expected = vec![
        OpCode::Jump as u8,
        6,
        OpCode::GetConstant as u8,
        0,
        OpCode::Mult as u8,
        OpCode::Return as u8,
        OpCode::GetConstant as u8,
        1,
        OpCode::MarkJump as u8,
        2,
        OpCode::GetConstant as u8,
        2,
        OpCode::MarkJump as u8,
        2,
        OpCode::Return as u8,
    ];
    let chunk = test_compile_tokens(input, expected);
    assert!(chunk.dictionary.contains_key("double"));
}

// TODO: recursive functions
#[test]
#[ignore = "when we have conditionals and built-in stack words"]
fn test_compile_recursive_functions() {
    let input = "
        : fib ( x -- y )
            dup 0 ==
            [ 1 ]
            [ dup 1 - fib swap 2 - fib + ] if ;
        4 fib
    ";
    let expected = vec![
        OpCode::Jump as u8,
        // fix this offset
        0,
        // What will go here for `dup`?
        OpCode::GetConstant as u8,
        0,
        OpCode::Equal as u8,
        // What will go here for the lambda?
        // What will go here for the lambda?
        // What will go here for `if`?
        OpCode::Return as u8,
        OpCode::GetConstant as u8,
        4,
        OpCode::MarkJump as u8,
        2,
        OpCode::Return as u8,
    ];
    test_compile_tokens(input, expected);
}

#[test]
fn test_compile_comments() {
    // env_logger::builder().init();
    // Basically this is just `test_compile_function_application`
    // with a comment added. The output of the compiler shouldn't
    // change at all.
    let input = "
        # This is a comment.
        ## This will be a documentation comment.
        ## With two lines.
        : double ( x -- y ) 2 * ;
        4 double
        5 double
    ";
    let expected = vec![
        OpCode::Jump as u8,
        6,
        OpCode::GetConstant as u8,
        0,
        OpCode::Mult as u8,
        OpCode::Return as u8,
        OpCode::GetConstant as u8,
        1,
        OpCode::MarkJump as u8,
        2,
        OpCode::GetConstant as u8,
        2,
        OpCode::MarkJump as u8,
        2,
        OpCode::Return as u8,
    ];
    let chunk = test_compile_tokens(input, expected);
    assert!(chunk.dictionary.contains_key("double"));
    let function = &chunk.dictionary["double"];
    assert_eq!(
        function.doc_comment,
        Some(" This will be a documentation comment.\n With two lines.\n".to_string())
    );
}

#[test]
fn test_compile_lambda() {
    // env_logger::builder().init();
    let input = "4 [ 2 * ] call";
    let expected = vec![
        OpCode::GetConstant as u8,
        0,
        OpCode::Jump as u8,
        8,
        OpCode::GetConstant as u8,
        1,
        OpCode::Mult as u8,
        OpCode::Return as u8,
        OpCode::GetConstant as u8,
        2,
        OpCode::CallTardiFn as u8,
        0,
        OpCode::Return as u8,
    ];

    let chunk = test_compile_tokens(input, expected);
    let function = &chunk.constants[2];

    assert!(matches!(function, Value::Lambda(_, _)));
    if let Value::Lambda(repr, jump) = function {
        assert_eq!(&"[ 2 * ]".to_string(), repr);
        assert_eq!(&4, jump);
    }
}
