use pretty_assertions::assert_eq;

use super::*;

fn test_parse_token_types(input: &str, expected: Vec<TokenType>) {
    let result = parse(input)
        .into_iter()
        .map(|t| t.map(|t| t.token_type))
        .collect::<Result<Vec<_>>>();

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result, expected);
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
        TokenType::Divide
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_plus() {
    let input = "10 3 +";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Plus
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_minus() {
    let input = "10 3 -";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Minus
    ];
    test_parse_token_types(input, expected);
}

#[test]
fn test_parse_multiply() {
    let input = "10 3 *";
    let expected = vec![
        TokenType::Integer(10),
        TokenType::Integer(3),
        TokenType::Multiply
    ];
    test_parse_token_types(input, expected);
}
