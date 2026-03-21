use crate::ast::Span;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { span: Span, message: String },
    UnexpectedEof { span: Span, context: String },
    UnclosedDelimiter { span: Span, delimiter: String },
    UnclosedDefinition { span: Span },
    InvalidTypeSig { span: Span, message: String },
}

impl ParseError {
    pub fn unexpected_token(span: Span, message: impl Into<String>) -> Self {
        ParseError::UnexpectedToken { span, message: message.into() }
    }

    pub fn unexpected_eof(span: Span, context: impl Into<String>) -> Self {
        ParseError::UnexpectedEof { span, context: context.into() }
    }

    pub fn unclosed_delimiter(span: Span, delimiter: impl Into<String>) -> Self {
        ParseError::UnclosedDelimiter { span, delimiter: delimiter.into() }
    }

    pub fn unclosed_definition(span: Span) -> Self {
        ParseError::UnclosedDefinition { span }
    }

    pub fn invalid_type_sig(span: Span, message: impl Into<String>) -> Self {
        ParseError::InvalidTypeSig { span, message: message.into() }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken { span, message } => {
                write!(f, "{}: {}", span, message)
            }
            ParseError::UnexpectedEof { span, context } => {
                write!(f, "{}: unexpected end of input ({})", span, context)
            }
            ParseError::UnclosedDelimiter { span, delimiter } => {
                write!(f, "{}: unclosed '{}' delimiter", span, delimiter)
            }
            ParseError::UnclosedDefinition { span } => {
                write!(f, "{}: unclosed ':' definition (missing ';')", span)
            }
            ParseError::InvalidTypeSig { span, message } => {
                write!(f, "{}: invalid type signature: {}", span, message)
            }
        }
    }
}

impl std::error::Error for ParseError {}
