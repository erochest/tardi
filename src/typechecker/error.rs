use crate::ast::Span;

#[derive(Debug, Clone)]
pub enum TypeError {
    UnknownWord { span: Span, name: String },
    TypeMismatch { span: Span, message: String },
    MissingEffect { span: Span, message: String },
    Internal { message: String },
}

impl TypeError {
    pub fn unknown_word(span: Span, name: impl Into<String>) -> Self {
        TypeError::UnknownWord { span, name: name.into() }
    }

    pub fn type_mismatch(span: Span, message: impl Into<String>) -> Self {
        TypeError::TypeMismatch { span, message: message.into() }
    }

    pub fn missing_effect(span: Span, message: impl Into<String>) -> Self {
        TypeError::MissingEffect { span, message: message.into() }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        TypeError::Internal { message: message.into() }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::UnknownWord { span, name } => {
                write!(f, "{}: unknown word '{}'", span, name)
            }
            TypeError::TypeMismatch { span, message } => {
                write!(f, "{}: type mismatch: {}", span, message)
            }
            TypeError::MissingEffect { span, message } => {
                write!(f, "{}: {}", span, message)
            }
            TypeError::Internal { message } => {
                write!(f, "internal type checker error: {}", message)
            }
        }
    }
}

impl std::error::Error for TypeError {}
