use crate::error::ScannerError;
use crate::vm::Value;
use std::convert::TryFrom;

/// Represents a token's type and any associated literal value
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Integer(i64),
    Float(f64),
    Boolean(bool),

    // Stack Operations
    Dup,
    Swap,
    Rot,
    Drop,

    // Special tokens
    Error,
    Eof,
}

/// Represents a token in the source code
#[derive(Debug, Clone)]
pub struct Token {
    /// The type of token and any associated literal value
    pub token_type: TokenType,

    /// Line number in source (1-based)
    pub line: usize,

    /// Column number in source (1-based)
    pub column: usize,

    /// Offset from start of source (0-based)
    pub offset: usize,

    /// Length of the token in characters
    pub length: usize,

    /// The actual text of the token from source
    pub lexeme: String,
}

impl Token {
    /// Creates a new token
    pub fn new(
        token_type: TokenType,
        line: usize,
        column: usize,
        offset: usize,
        length: usize,
        lexeme: String,
    ) -> Self {
        Token {
            token_type,
            line,
            column,
            offset,
            length,
            lexeme,
        }
    }
}

impl TryFrom<Token> for Value {
    type Error = ScannerError;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Integer(n) => Ok(Value::Integer(n)),
            TokenType::Float(n) => Ok(Value::Float(n)),
            TokenType::Boolean(b) => Ok(Value::Boolean(b)),
            _ => Err(ScannerError::InvalidLiteral(token.lexeme)),
        }
    }
}

// TODO: refactor into a new file
#[cfg(test)]
mod tests {
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
}
