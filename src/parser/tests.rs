use pretty_assertions::assert_eq;

use super::*;

#[test]
fn test_parse_integer() {
    let input = "10 3";
    let expected = vec![TokenType::Integer(10), TokenType::Integer(3)];
    let result = parse(input)
        .into_iter()
        .map(|t| t.map(|t| t.token_type))
        .collect::<Result<Vec<_>>>();

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_plus() {
    let input = "10 3 +";
    let expected = vec![TokenType::Integer(10), TokenType::Integer(3), TokenType::Plus];
    let result = parse(input)
        .into_iter()
        .map(|t| t.map(|t| t.token_type))
        .collect::<Result<Vec<_>>>();

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parse_minus() {
    let input = "10 3 -";
    let expected = vec![TokenType::Integer(10), TokenType::Integer(3), TokenType::Minus];
    let result = parse(input)
        .into_iter()
        .map(|t| t.map(|t| t.token_type))
        .collect::<Result<Vec<_>>>();
}