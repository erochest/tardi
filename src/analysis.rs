//! High-level utilities for the parser + type-checker pipeline.
//!
//! The functions here run the new analysis pipeline on raw source strings:
//!
//! ```text
//! source  →  Scanner  →  Parser  →  AstNode
//!                                      ↓
//!                              TypeChecker  →  annotated AstNode
//! ```
//!
//! This pipeline is independent of the existing compiler back-end and is the
//! foundation for future tooling (LSP, native-code backends, etc.).

use std::collections::HashMap;

use crate::ast::AstNode;
use crate::parser::{ParseError, Parser};
use crate::scanner::Scanner;
use crate::typechecker::{TypeChecker, TypeError};
use crate::types::TypeSig;

/// Error returned by the analysis pipeline.
#[derive(Debug, Clone)]
pub enum AnalysisError {
    /// The token stream could not be parsed into an AST.
    Parse(ParseError),
    /// The AST did not pass type checking.
    Type(TypeError),
}

impl From<ParseError> for AnalysisError {
    fn from(e: ParseError) -> Self {
        AnalysisError::Parse(e)
    }
}

impl From<TypeError> for AnalysisError {
    fn from(e: TypeError) -> Self {
        AnalysisError::Type(e)
    }
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::Parse(e) => write!(f, "parse error: {}", e),
            AnalysisError::Type(e) => write!(f, "type error: {}", e),
        }
    }
}

impl std::error::Error for AnalysisError {}

/// Scan and parse `source` into an un-annotated `Program` AST node.
///
/// Scanner errors propagate as `AnalysisError::Parse`.
pub fn parse_source(source: &str, source_id: &str) -> Result<AstNode, AnalysisError> {
    let mut scanner = Scanner::from_input_string(source);
    let tokens = scanner
        .scan_to_end()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            AnalysisError::Parse(ParseError::unexpected_token(
                crate::ast::Span::unknown(source_id.to_string()),
                format!("scanner error: {}", e),
            ))
        })?;

    let mut parser = Parser::new(tokens, source_id);
    Ok(parser.parse()?)
}

/// Scan, parse, and type-check `source`, returning a fully annotated AST.
///
/// `env` maps word names to their declared `TypeSig` and is used to resolve
/// word references during type checking.  Pass an empty map if you only want
/// to check a self-contained snippet.
pub fn check_source(
    source: &str,
    source_id: &str,
    env: HashMap<String, TypeSig>,
) -> Result<AstNode, AnalysisError> {
    let mut ast = parse_source(source, source_id)?;
    let mut checker = TypeChecker::with_env(env);
    checker.check(&mut ast)?;
    Ok(ast)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::ast::NodeKind;
    use crate::types::TypeSig;

    fn sig(s: &str) -> TypeSig {
        TypeSig::from_str(s).unwrap()
    }

    // ── parse_source ──────────────────────────────────────────────────────────

    #[test]
    fn parse_integer_literal() {
        let ast = parse_source("42", "test").unwrap();
        assert!(matches!(ast.kind, NodeKind::Program(ref items) if items.len() == 1));
    }

    #[test]
    fn parse_definition() {
        let ast = parse_source(": double ( int -- int ) dup + ;", "test").unwrap();
        match &ast.kind {
            NodeKind::Program(items) => {
                assert_eq!(items.len(), 1);
                assert!(
                    matches!(&items[0].kind, NodeKind::Definition { name, .. } if name == "double")
                );
            }
            _ => panic!("expected Program"),
        }
    }

    #[test]
    fn parse_quotation() {
        let ast = parse_source("[ 1 2 + ]", "test").unwrap();
        match &ast.kind {
            NodeKind::Program(items) => {
                assert_eq!(items.len(), 1);
                assert!(matches!(&items[0].kind, NodeKind::Quotation(_)));
            }
            _ => panic!("expected Program"),
        }
    }

    #[test]
    fn parse_vector() {
        let ast = parse_source("{ 1 2 3 }", "test").unwrap();
        match &ast.kind {
            NodeKind::Program(items) => {
                assert!(matches!(&items[0].kind, NodeKind::Vector(_)));
            }
            _ => panic!("expected Program"),
        }
    }

    #[test]
    fn parse_error_unclosed_quotation() {
        let result = parse_source("[ 1 2", "test");
        assert!(matches!(result, Err(AnalysisError::Parse(_))));
    }

    // ── check_source ──────────────────────────────────────────────────────────

    #[test]
    fn check_well_typed_definition() {
        let mut env = HashMap::new();
        env.insert("dup".to_string(), sig("( a -- a a )"));
        env.insert("+".to_string(), sig("( int int -- int )"));

        let result = check_source(": double ( int -- int ) dup + ;", "test", env);
        assert!(result.is_ok(), "{:?}", result);

        // Definition node should be annotated
        let ast = result.unwrap();
        match &ast.kind {
            NodeKind::Program(items) => {
                assert!(items[0].type_info.is_some());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn check_type_error_unknown_word() {
        let result = check_source("undefined-word", "test", HashMap::new());
        assert!(matches!(result, Err(AnalysisError::Type(_))));
    }

    #[test]
    fn check_effect_error_undeclared() {
        let mut env = HashMap::new();
        env.insert("print".to_string(), sig("( str -- | io )"));

        // Definition uses `print` (which has io effect) but doesn't declare it
        let result = check_source(": silent-print ( str -- ) print ;", "test", env);
        assert!(matches!(result, Err(AnalysisError::Type(_))));
    }

    #[test]
    fn check_forward_references() {
        let mut env = HashMap::new();
        env.insert("+".to_string(), sig("( int int -- int )"));

        // bar references foo; both defined in same snippet
        let result = check_source(
            ": bar ( -- int ) foo ; \
             : foo ( -- int ) 42 ;",
            "test",
            env,
        );
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    fn parse_and_check_nested_quotation() {
        let mut env = HashMap::new();
        env.insert("apply".to_string(), sig("( S a -- S | effect )"));

        let result = check_source("[ 1 ] apply", "test", env);
        // Top-level expressions (not in a definition) may have unknown words;
        // this verifies parsing and partial type checking work on quotations.
        // `1` is typed, `apply` is in env, so this should succeed.
        assert!(result.is_ok(), "{:?}", result);
    }
}
