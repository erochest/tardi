use super::*;

#[test]
fn test_token_creation() {
    let token = Token::new(TokenType::Integer(42), 1, 10, 9, 2, "42".to_string());

    assert_eq!(token.line, 1);
    assert_eq!(token.column, 10);
    assert_eq!(token.offset, 9);
    assert_eq!(token.length, 2);
    assert_eq!(token.lexeme, "42");
    assert!(matches!(token.token_type, TokenType::Integer(42)));
}

#[test]
fn test_token_to_value_conversion() {
    let token = Token::new(TokenType::Integer(42), 1, 1, 0, 2, "42".to_string());
    let value = Value::try_from(token).unwrap();
    assert!(matches!(value, Value::Integer(42)));

    let token = Token::new(TokenType::Float(3.14), 1, 1, 0, 4, "3.14".to_string());
    let value = Value::try_from(token).unwrap();
    assert!(matches!(value, Value::Float(3.14)));

    let token = Token::new(TokenType::Boolean(true), 1, 1, 0, 2, "#t".to_string());
    let value = Value::try_from(token).unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let token = Token::new(TokenType::Boolean(false), 1, 1, 0, 2, "#f".to_string());
    let value = Value::try_from(token).unwrap();
    assert!(matches!(value, Value::Boolean(false)));

    let token = Token::new(TokenType::Error, 1, 1, 0, 5, "error".to_string());
    assert!(Value::try_from(token).is_err());
}
