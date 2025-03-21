use crate::error::ScannerError;
use crate::vm::value::Value;
use std::convert::TryFrom;

/// Represents a token's type and any associated literal value
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),

    // Stack Operations
    Dup,
    Swap,
    Rot,
    Drop,

    // Return Stack Operations
    ToR,    // >r
    RFrom,  // r>
    RFetch, // r@

    // Arithmetic Operators
    Plus,
    Dash,
    Star,
    Slash,

    // Comparison Operators
    EqualEqual,   // ==
    BangEqual,    // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Bang,         // !

    // Words
    Word(String),

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
            TokenType::Char(c) => Ok(Value::Char(c)),
            _ => Err(ScannerError::InvalidLiteral(token.lexeme)),
        }
    }
}

// TODO: refactor into a new file
#[cfg(test)]
mod tests;
