mod error;
use std::str::FromStr;

pub use error::ParseError;

use crate::ast::{AstNode, NodeKind, Span};
use crate::types::TypeSig;
use crate::value::{Value, ValueData};

/// Converts a raw scanner token stream into a Tardi AST.
///
/// The parser handles structural constructs directly, replacing the macros
/// that previously handled them in Pass 1:
///   - `[ ... ]`       → Quotation node
///   - `{ ... }`       → Vector node
///   - `: name sig ;`  → Definition node
///   - `\ token`       → Literal node (escaped token)
///   - `MACRO: name ;` → MacroDefinition node (body compiled later)
///
/// Everything else becomes a Word or Literal node.
pub struct Parser {
    tokens: Vec<Value>,
    pos: usize,
    source_id: String,
}

impl Parser {
    pub fn new(tokens: Vec<Value>, source_id: impl Into<String>) -> Self {
        Parser {
            tokens,
            pos: 0,
            source_id: source_id.into(),
        }
    }

    /// Parse the entire token stream into a `Program` node.
    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        let prog_span = Span::unknown(self.source_id.clone());
        let mut items = Vec::new();
        while self.pos < self.tokens.len() {
            if let Some(node) = self.parse_item()? {
                items.push(node);
            }
        }
        Ok(AstNode::new(NodeKind::Program(items), prog_span))
    }

    // ── Token access helpers ──────────────────────────────────────────────────

    fn peek(&self) -> Option<&Value> {
        self.tokens.get(self.pos)
    }

    fn peek_word(&self) -> Option<&str> {
        self.peek().and_then(|v| match &v.data {
            ValueData::Word(w) => Some(w.as_str()),
            ValueData::Symbol { word, .. } => Some(word.as_str()),
            _ => None,
        })
    }

    fn current_span(&self) -> Span {
        self.peek()
            .and_then(|v| {
                v.pos
                    .as_ref()
                    .map(|p| Span::from_pos(self.source_id.clone(), p))
            })
            .unwrap_or_else(|| Span::unknown(self.source_id.clone()))
    }

    fn value_span(&self, value: &Value) -> Span {
        value
            .pos
            .as_ref()
            .map(|p| Span::from_pos(self.source_id.clone(), p))
            .unwrap_or_else(|| Span::unknown(self.source_id.clone()))
    }

    /// Advance past the current token, returning a clone of it.
    fn advance(&mut self) -> Option<Value> {
        if self.pos < self.tokens.len() {
            let v = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(v)
        } else {
            None
        }
    }

    // ── Item dispatch ─────────────────────────────────────────────────────────

    fn parse_item(&mut self) -> Result<Option<AstNode>, ParseError> {
        let value = match self.peek() {
            None => return Ok(None),
            Some(v) => v.clone(),
        };

        let span = self.value_span(&value);

        match &value.data {
            // ── Literals ──────────────────────────────────────────────────────
            ValueData::Integer(_)
            | ValueData::Float(_)
            | ValueData::Boolean(_)
            | ValueData::Char(_)
            | ValueData::String(_) => {
                self.advance();
                Ok(Some(AstNode::new(NodeKind::Literal(value), span)))
            }

            // ── Symbols (the scanner emits all words as Symbol tokens) ────────
            //
            // The scanner's `parse_word` always produces `Symbol { module, word }`
            // rather than `Word(w)`.  We dispatch structural keywords here just as
            // we do in the `Word` arm, and treat everything else as an unqualified
            // `Word` node (module resolution is a future concern).
            ValueData::Symbol { word, .. } => {
                let w = word.clone();
                match w.as_str() {
                    "[" => {
                        self.advance();
                        self.parse_quotation(span)
                    }
                    "{" => {
                        self.advance();
                        self.parse_vector(span)
                    }
                    ":" => {
                        self.advance();
                        self.parse_definition(span)
                    }
                    "\\" => {
                        self.advance();
                        self.parse_escape(span)
                    }

                    "]" | "}" | ";" => Err(ParseError::unexpected_token(
                        span,
                        format!("unexpected closing delimiter '{}'", w),
                    )),

                    _ => {
                        self.advance();
                        Ok(Some(AstNode::new(
                            NodeKind::Word {
                                module: None,
                                name: w,
                            },
                            span,
                        )))
                    }
                }
            }

            // ── MACRO: keyword ────────────────────────────────────────────────
            ValueData::Macro => {
                self.advance();
                self.parse_macro_definition(span)
            }

            // ── Words — main structural dispatch ──────────────────────────────
            ValueData::Word(w) => {
                let w = w.clone();
                match w.as_str() {
                    "[" => {
                        self.advance();
                        self.parse_quotation(span)
                    }
                    "{" => {
                        self.advance();
                        self.parse_vector(span)
                    }
                    ":" => {
                        self.advance();
                        self.parse_definition(span)
                    }
                    "\\" => {
                        self.advance();
                        self.parse_escape(span)
                    }

                    // Closing delimiters at the top level are errors
                    "]" | "}" | ";" => Err(ParseError::unexpected_token(
                        span,
                        format!("unexpected closing delimiter '{}'", w),
                    )),

                    _ => {
                        self.advance();
                        Ok(Some(AstNode::new(
                            NodeKind::Word {
                                module: None,
                                name: w,
                            },
                            span,
                        )))
                    }
                }
            }

            // ── Anything else (e.g. Function values from pre-parsed streams) ──
            _ => {
                self.advance();
                Ok(Some(AstNode::new(NodeKind::Literal(value), span)))
            }
        }
    }

    // ── Structural parsers ────────────────────────────────────────────────────

    /// Parse `[ body... ]` into a Quotation node.
    fn parse_quotation(&mut self, open_span: Span) -> Result<Option<AstNode>, ParseError> {
        let mut body = Vec::new();
        loop {
            if self.peek().is_none() {
                return Err(ParseError::unclosed_delimiter(open_span, "["));
            }
            match self.peek_word() {
                Some("]") => {
                    let close_span = self.current_span();
                    self.advance();
                    return Ok(Some(AstNode::new(
                        NodeKind::Quotation(body),
                        Span::merge(&open_span, &close_span),
                    )));
                }
                _ => {
                    if let Some(node) = self.parse_item()? {
                        body.push(node);
                    }
                }
            }
        }
    }

    /// Parse `{ items... }` into a Vector node.
    fn parse_vector(&mut self, open_span: Span) -> Result<Option<AstNode>, ParseError> {
        let mut items = Vec::new();
        loop {
            if self.peek().is_none() {
                return Err(ParseError::unclosed_delimiter(open_span, "{"));
            }
            match self.peek_word() {
                Some("}") => {
                    let close_span = self.current_span();
                    self.advance();
                    return Ok(Some(AstNode::new(
                        NodeKind::Vector(items),
                        Span::merge(&open_span, &close_span),
                    )));
                }
                _ => {
                    if let Some(node) = self.parse_item()? {
                        items.push(node);
                    }
                }
            }
        }
    }

    /// Parse `: name ( type-sig ) body... ;` into a Definition node.
    ///
    /// The type signature `( ... )` is optional — definitions without one get
    /// a placeholder `( -- )` signature that the type checker will flag later.
    fn parse_definition(&mut self, colon_span: Span) -> Result<Option<AstNode>, ParseError> {
        // Read the word name
        let name_value = self
            .advance()
            .ok_or_else(|| ParseError::unexpected_eof(colon_span.clone(), "word name after ':'"))?;
        let name = match &name_value.data {
            ValueData::Word(w) => w.clone(),
            ValueData::Symbol { word, .. } => word.clone(),
            other => {
                return Err(ParseError::unexpected_token(
                    self.value_span(&name_value),
                    format!("expected word name after ':', got {:?}", other),
                ))
            }
        };

        // Optionally parse type signature `( ... )`
        let type_sig = if self.peek_word() == Some("(") {
            self.parse_type_sig()?
        } else {
            TypeSig::from_str("( -- )").unwrap()
        };

        // Parse the body until `;`
        let mut body = Vec::new();
        loop {
            if self.peek().is_none() {
                return Err(ParseError::unclosed_definition(colon_span));
            }
            match self.peek_word() {
                Some(";") => {
                    let close_span = self.current_span();
                    self.advance();
                    return Ok(Some(AstNode::new(
                        NodeKind::Definition {
                            name,
                            type_sig,
                            body,
                        },
                        Span::merge(&colon_span, &close_span),
                    )));
                }
                _ => {
                    if let Some(node) = self.parse_item()? {
                        body.push(node);
                    }
                }
            }
        }
    }

    /// Parse `( inputs -- outputs | effects )` into a TypeSig.
    fn parse_type_sig(&mut self) -> Result<TypeSig, ParseError> {
        let open_span = self.current_span();
        self.advance(); // consume `(`

        let mut parts = vec!["(".to_string()];
        let mut depth: i32 = 1;

        loop {
            match self.advance() {
                None => return Err(ParseError::unclosed_delimiter(open_span, "(")),
                Some(value) => {
                    // The scanner emits both `Word` and `Symbol` variants; treat
                    // them uniformly — for type signatures we just want the raw text.
                    let text = match &value.data {
                        ValueData::Word(w) => {
                            match w.as_str() {
                                "(" => {
                                    depth += 1;
                                }
                                ")" => {
                                    depth -= 1;
                                }
                                _ => {}
                            }
                            w.clone()
                        }
                        ValueData::Symbol { word, .. } => {
                            match word.as_str() {
                                "(" => {
                                    depth += 1;
                                }
                                ")" => {
                                    depth -= 1;
                                }
                                _ => {}
                            }
                            word.clone()
                        }
                        other => format!("{}", other),
                    };
                    parts.push(text);
                    if depth == 0 {
                        break;
                    }
                }
            }
        }

        let sig_str = parts.join(" ");
        TypeSig::from_str(&sig_str).map_err(|e| ParseError::invalid_type_sig(open_span, e.0))
    }

    /// Parse `\ token` — the next token becomes a Literal node.
    fn parse_escape(&mut self, backslash_span: Span) -> Result<Option<AstNode>, ParseError> {
        let value = self.advance().ok_or_else(|| {
            ParseError::unexpected_eof(backslash_span.clone(), "token after '\\'")
        })?;
        let token_span = self.value_span(&value);
        let span = Span::merge(&backslash_span, &token_span);
        Ok(Some(AstNode::new(NodeKind::Literal(value), span)))
    }

    /// Parse `MACRO: name body... ;` into a MacroDefinition node.
    /// The body is kept as raw tokens for the compiler to compile and register.
    fn parse_macro_definition(&mut self, macro_span: Span) -> Result<Option<AstNode>, ParseError> {
        // Read the trigger word
        let name_value = self.advance().ok_or_else(|| {
            ParseError::unexpected_eof(macro_span.clone(), "macro name after 'MACRO:'")
        })?;
        let name = match &name_value.data {
            ValueData::Word(w) => w.clone(),
            ValueData::Symbol { word, .. } => word.clone(),
            _ => {
                return Err(ParseError::unexpected_token(
                    self.value_span(&name_value),
                    "expected macro name after 'MACRO:'",
                ))
            }
        };

        // Collect raw tokens until `;`
        let mut body: Vec<Value> = Vec::new();
        loop {
            if self.peek().is_none() {
                return Err(ParseError::unclosed_definition(macro_span.clone()));
            }
            match self.peek_word() {
                Some(";") => {
                    let close_span = self.current_span();
                    self.advance();
                    return Ok(Some(AstNode::new(
                        NodeKind::MacroDefinition { name, body },
                        Span::merge(&macro_span, &close_span),
                    )));
                }
                _ => {
                    body.push(self.advance().unwrap());
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ValueData;

    fn parse(src: &str) -> AstNode {
        use crate::scanner::Scanner;
        let mut scanner = Scanner::from_input_string(src);
        let tokens = scanner
            .scan_to_end()
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        let mut parser = Parser::new(tokens, "test");
        parser.parse().expect("parse error")
    }

    fn program_items(node: AstNode) -> Vec<AstNode> {
        match node.kind {
            NodeKind::Program(items) => items,
            other => panic!("expected Program, got {:?}", other),
        }
    }

    #[test]
    fn parse_integer_literal() {
        let items = program_items(parse("42"));
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, NodeKind::Literal(v) if v.data == ValueData::Integer(42)));
    }

    #[test]
    fn parse_string_literal() {
        let items = program_items(parse("\"hello\""));
        assert_eq!(items.len(), 1);
        assert!(
            matches!(&items[0].kind, NodeKind::Literal(v) if v.data == ValueData::String("hello".to_string()))
        );
    }

    #[test]
    fn parse_word() {
        let items = program_items(parse("dup"));
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, NodeKind::Word { module: None, name } if name == "dup"));
    }

    #[test]
    fn parse_sequence() {
        let items = program_items(parse("1 2 +"));
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn parse_empty_quotation() {
        let items = program_items(parse("[ ]"));
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, NodeKind::Quotation(body) if body.is_empty()));
    }

    #[test]
    fn parse_quotation_with_body() {
        let items = program_items(parse("[ 1 2 + ]"));
        assert_eq!(items.len(), 1);
        match &items[0].kind {
            NodeKind::Quotation(body) => assert_eq!(body.len(), 3),
            other => panic!("expected Quotation, got {:?}", other),
        }
    }

    #[test]
    fn parse_nested_quotation() {
        let items = program_items(parse("[ [ 1 ] ]"));
        assert_eq!(items.len(), 1);
        match &items[0].kind {
            NodeKind::Quotation(body) => {
                assert_eq!(body.len(), 1);
                assert!(matches!(&body[0].kind, NodeKind::Quotation(_)));
            }
            other => panic!("expected Quotation, got {:?}", other),
        }
    }

    #[test]
    fn parse_empty_vector() {
        let items = program_items(parse("{ }"));
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, NodeKind::Vector(v) if v.is_empty()));
    }

    #[test]
    fn parse_vector_with_literals() {
        let items = program_items(parse("{ 1 2 3 }"));
        assert_eq!(items.len(), 1);
        match &items[0].kind {
            NodeKind::Vector(items) => assert_eq!(items.len(), 3),
            other => panic!("expected Vector, got {:?}", other),
        }
    }

    #[test]
    fn parse_definition_no_type_sig() {
        let items = program_items(parse(": double   dup + ;"));
        assert_eq!(items.len(), 1);
        match &items[0].kind {
            NodeKind::Definition { name, body, .. } => {
                assert_eq!(name, "double");
                assert_eq!(body.len(), 2);
            }
            other => panic!("expected Definition, got {:?}", other),
        }
    }

    #[test]
    fn parse_definition_with_type_sig() {
        let items = program_items(parse(": double ( S int -- S int )   dup + ;"));
        assert_eq!(items.len(), 1);
        match &items[0].kind {
            NodeKind::Definition {
                name,
                type_sig,
                body,
            } => {
                assert_eq!(name, "double");
                assert_eq!(body.len(), 2);
                assert!(type_sig.to_string().contains("int"));
            }
            other => panic!("expected Definition, got {:?}", other),
        }
    }

    #[test]
    fn parse_escape() {
        let items = program_items(parse("\\ ;"));
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0].kind, NodeKind::Literal(_)));
    }

    #[test]
    fn parse_macro_definition() {
        let items = program_items(parse("MACRO: my-macro   dup swap ;"));
        assert_eq!(items.len(), 1);
        match &items[0].kind {
            NodeKind::MacroDefinition { name, body } => {
                assert_eq!(name, "my-macro");
                assert_eq!(body.len(), 2);
            }
            other => panic!("expected MacroDefinition, got {:?}", other),
        }
    }

    #[test]
    fn parse_definition_containing_quotation() {
        let items = program_items(parse(": dip   swap >r apply r> ;"));
        match &items[0].kind {
            NodeKind::Definition { name, body, .. } => {
                assert_eq!(name, "dip");
                assert_eq!(body.len(), 4);
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn span_is_present_on_literal() {
        let items = program_items(parse("99"));
        // Span should have a real position (not all zeros from unknown())
        assert_eq!(items[0].span.start.line, 1);
    }

    #[test]
    fn unclosed_quotation_is_error() {
        use crate::scanner::Scanner;
        let mut scanner = Scanner::from_input_string("[ 1 2");
        let tokens = scanner
            .scan_to_end()
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        let mut parser = Parser::new(tokens, "test");
        assert!(parser.parse().is_err());
    }

    #[test]
    fn unclosed_definition_is_error() {
        use crate::scanner::Scanner;
        let mut scanner = Scanner::from_input_string(": foo 1 2");
        let tokens = scanner
            .scan_to_end()
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
        let mut parser = Parser::new(tokens, "test");
        assert!(parser.parse().is_err());
    }
}
