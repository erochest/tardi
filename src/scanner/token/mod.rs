use crate::error::{CompilerError, Error, ScannerError};
use crate::value::Value;
use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::result;

/// Represents a token's type and any associated literal value
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TokenType {
    // Literals
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),
    String(String),

    // Stack Operations
    Dup,
    Swap,
    Rot,
    Drop,
    StackSize,

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
    MacroStart, // MACRO:
    Lit,

    // List Operations
    CreateList, // <list>
    Append,     // append
    Prepend,    // prepend
    Concat,     // concat
    SplitHead,  // split-head!

    // String Operations
    CreateString, // <string>
    ToString,     // >string
    Utf8ToString, // utf8>string
    StringConcat, // string-concat

    // Function Operations
    Function, // <function>
    Lambda,   // <lambda>
    Call,     // call

    // Delimiters
    LeftCurly,  // {
    RightCurly, // }

    // Scanning Ops
    ScanToken,
    ScanTokenList,
    ScanValueList,

    // Special tokens
    Error,
    EndOfInput,
}

fn format_char(c: char) -> String {
    match c {
        '\n' => "\n".to_string(),
        '\r' => "\r".to_string(),
        '\t' => "\t".to_string(),
        c => c.to_string(),
    }
}

fn format_str(str: &str) -> String {
    str.chars().map(format_char).collect::<Vec<_>>().join("")
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Integer(i) => write!(f, "{}", i),
            TokenType::Float(d) => write!(f, "{}", d),
            TokenType::Boolean(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            TokenType::Char(c) => write!(f, "'{}'", format_char(*c)),
            TokenType::String(str) => write!(f, "\"{}\"", format_str(str)),
            TokenType::Dup => write!(f, "dup"),
            TokenType::Swap => write!(f, "swap"),
            TokenType::Rot => write!(f, "rot"),
            TokenType::Drop => write!(f, "drop"),
            TokenType::StackSize => write!(f, "stack-size"),
            TokenType::ToR => write!(f, ">r"),
            TokenType::RFrom => write!(f, "r>"),
            TokenType::RFetch => write!(f, "r@"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Dash => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Less => write!(f, "<"),
            TokenType::Greater => write!(f, ">"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Bang => write!(f, "!"),
            TokenType::Word(word) => write!(f, "{}", word),
            TokenType::MacroStart => write!(f, "MACRO:"),
            TokenType::Lit => write!(f, "lit"),
            TokenType::CreateList => write!(f, "<list>"),
            TokenType::Append => write!(f, "append"),
            TokenType::Prepend => write!(f, "prepend"),
            TokenType::Concat => write!(f, "concat"),
            TokenType::SplitHead => write!(f, "split-head!"),
            TokenType::CreateString => write!(f, "<string>"),
            TokenType::ToString => write!(f, ">string"),
            TokenType::Utf8ToString => write!(f, "utf8>string"),
            TokenType::StringConcat => write!(f, "string-concat"),
            TokenType::Function => write!(f, "<function>"),
            TokenType::Lambda => write!(f, "<lambda>"),
            TokenType::Call => write!(f, "call"),
            TokenType::LeftCurly => write!(f, "{{"),
            TokenType::RightCurly => write!(f, "}}"),
            TokenType::ScanToken => write!(f, "scan-token"),
            TokenType::ScanTokenList => write!(f, "scan-token-list"),
            TokenType::ScanValueList => write!(f, "scan-value-list"),
            TokenType::Error => write!(f, "<error>"),
            TokenType::EndOfInput => write!(f, "<end-of-input>"),
        }
    }
}

impl TryFrom<Value> for TokenType {
    type Error = Error;

    fn try_from(value: Value) -> result::Result<Self, Error> {
        match value {
            Value::Integer(v) => Ok(TokenType::Integer(v)),
            Value::Float(v) => Ok(TokenType::Float(v)),
            Value::Boolean(v) => Ok(TokenType::Boolean(v)),
            Value::Char(v) => Ok(TokenType::Char(v)),
            Value::List(vec) => {
                Err(CompilerError::ValueHasNoTokenType(format!("{}", Value::List(vec))).into())
            }
            Value::String(s) => Ok(TokenType::String(s)),
            Value::Function(callable) => Ok(TokenType::Word(
                callable
                    .get_name()
                    .unwrap_or_else(|| "<lambda>".to_string()),
            )),
            Value::Address(v) => Ok(TokenType::Integer(v as i64)),
            Value::Token(token) => Ok(token.token_type),
            Value::Literal(value) => TokenType::try_from(*value),
        }
    }
}

/// Represents a token in the source code
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token_type)
    }
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
            TokenType::String(s) => Ok(Value::String(s)),
            _ => Err(ScannerError::InvalidLiteral(token.lexeme)),
        }
    }
}

// TODO: refactor into a new file
#[cfg(test)]
mod tests;
