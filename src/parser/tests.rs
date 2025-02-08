use std::iter::Iterator;

use pretty_assertions::assert_eq;

use super::*;

fn test_parse_token_types(input: &str, expected: Vec<TokenType>) {
    let result = parse(input).map(|t| t.into_iter().map(|t| t.token_type).collect::<Vec<_>>());

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_skips_whitespace() {
    let input = "\n1\n";
    let expected = vec![TokenType::Integer(1)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_with_escaped_whitespace() {
    let input = "\"\\tthis string contains\\nwhitespace\\n\\tlots of whitespace.\\r\\n\"";
    let expected = vec![TokenType::String(
        "\tthis string contains\nwhitespace\n\tlots of whitespace.\r\n".to_string(),
    )];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_with_escaped_quotes() {
    let input = "\"the alien said, \\\"greetings, earthling.\\\"\"";
    let expected = vec![TokenType::String(
        "the alien said, \"greetings, earthling.\"".to_string(),
    )];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_multiple_words() {
    let input = "\"hello world\"";
    let expected = vec![TokenType::String("hello world".to_string())];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_integer() {
    let input = "10 3";
    let expected = vec![TokenType::Integer(10), TokenType::Integer(3)];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_divide() {
    let input = "10 3 /";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Division,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_plus() {
    let input = "10 3 +";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Plus,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_minus() {
    let input = "10 3 -";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Minus,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_multiply() {
    let input = "10 3 *";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Multiply,
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_empty() {
    let input = "\"\"";
    let expected = vec![TokenType::String(String::new())];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_string_single_word() {
    let input = "\"hello\"";
    let expected = vec![TokenType::String("hello".to_string())];
    test_parse_token_types(input, expected);
}
