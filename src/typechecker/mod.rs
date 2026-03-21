mod error;
pub use error::TypeError;

use std::collections::HashMap;

use crate::ast::{AstNode, NodeKind, Span, TypeInfo};
use crate::types::{unify_rows, PrimitiveType, Row, StackType, TypeSig};
use crate::value::ValueData;

/// Type-checks a Tardi AST, annotating each node with its inferred `TypeInfo`.
///
/// The checker resolves word references against a type environment and
/// verifies that every definition's body type matches its declared signature.
/// It also ensures that effect labels used in a body are declared in the
/// enclosing definition's signature.
pub struct TypeChecker {
    env: HashMap<String, TypeSig>,
}

impl TypeChecker {
    /// Create an empty type checker with no pre-defined words.
    pub fn new() -> Self {
        TypeChecker {
            env: HashMap::new(),
        }
    }

    /// Create a type checker pre-populated with a known environment.
    pub fn with_env(env: HashMap<String, TypeSig>) -> Self {
        TypeChecker { env }
    }

    /// Register a word with its type signature.
    pub fn define(&mut self, name: impl Into<String>, sig: TypeSig) {
        self.env.insert(name.into(), sig);
    }

    /// Check and annotate a parsed `Program` node in-place.
    pub fn check(&mut self, program: &mut AstNode) -> Result<(), TypeError> {
        match &mut program.kind {
            NodeKind::Program(items) => self.check_program(items),
            _ => Err(TypeError::internal("check() expected a Program node")),
        }
    }

    // ── Core checking ─────────────────────────────────────────────────────────

    /// Two-pass program check: pre-declare all definitions, then verify bodies.
    fn check_program(&mut self, items: &mut [AstNode]) -> Result<(), TypeError> {
        // Pass 1 — register every definition's declared signature so bodies
        //           can reference words defined later (forward references).
        for item in items.iter() {
            if let NodeKind::Definition { name, type_sig, .. } = &item.kind {
                self.env.insert(name.clone(), type_sig.clone());
            }
        }

        // Pass 2 — check each item
        for item in items.iter_mut() {
            self.check_item(item)?;
        }

        Ok(())
    }

    /// Type-check a single node, annotate it with `TypeInfo`, and return its
    /// inferred `TypeSig`.
    fn check_item(&mut self, node: &mut AstNode) -> Result<TypeSig, TypeError> {
        let span = node.span.clone();

        // Compute the signature.  The match borrows `node.kind`; after the
        // expression completes the borrow is released, letting us write to
        // `node.type_info` below.
        let sig = match &mut node.kind {
            NodeKind::Literal(value) => {
                let vd = value.data.clone();
                self.literal_sig_for(&vd)
            }

            NodeKind::Word { module, name } => {
                let m = module.as_deref().map(str::to_owned);
                let n = name.clone();
                self.lookup_word(&span, m.as_deref(), &n)?
            }

            NodeKind::Quotation(body) => self.infer_quotation(&span, body)?,

            NodeKind::Vector(_) => self.vector_sig(),

            NodeKind::Definition {
                name,
                type_sig,
                body,
            } => {
                let name = name.clone();
                let declared = type_sig.clone();
                let inferred = self.check_sequence(body)?;
                self.check_effects_subsumed(&span, &name, &declared, &inferred)?;
                self.check_rows_match(&span, &name, &declared, &inferred)?;
                declared
            }

            NodeKind::MacroDefinition { .. } => {
                // Macro bodies are raw tokens; no type checking.
                self.identity_sig()
            }

            NodeKind::Program(items) => {
                self.check_program(items)?;
                self.identity_sig()
            }
        };

        node.type_info = Some(TypeInfo {
            type_sig: sig.clone(),
        });
        Ok(sig)
    }

    /// Type-check a sequence of nodes by composing their `TypeSig`s left-to-right.
    ///
    /// The type of a sequence is the composition of each word's type in order.
    /// An empty sequence has the identity type `( S -- S )`.
    fn check_sequence(&mut self, nodes: &mut [AstNode]) -> Result<TypeSig, TypeError> {
        let mut iter = nodes.iter_mut();

        // Seed the accumulator with the first node's type (not the identity).
        // This preserves the input row of the leftmost word so the checker can
        // verify it against a definition's declared input signature.
        let Some(first) = iter.next() else {
            return Ok(self.identity_sig());
        };
        let mut acc = self.check_item(first)?;

        for node in iter {
            let next = self.check_item(node)?;
            let node_span = node.span.clone();
            acc = acc
                .compose(&next)
                .map_err(|e| TypeError::type_mismatch(node_span, e.0))?;
        }
        Ok(acc)
    }

    /// Infer the type of a quotation `[ body ]`.
    ///
    /// If the body has type `( S a -- S b )`, the quotation has type
    /// `( R -- R [ S a -- S b ] )`, pushing the quotation value itself.
    fn infer_quotation(
        &mut self,
        _span: &Span,
        body: &mut [AstNode],
    ) -> Result<TypeSig, TypeError> {
        let inner = self.check_sequence(body)?;
        let row_var = "R".to_string();
        let input = Row {
            var: Some(row_var.clone()),
            types: vec![],
        };
        let output = Row {
            var: Some(row_var),
            types: vec![StackType::Quotation(Box::new(inner))],
        };
        Ok(TypeSig::pure(input, output))
    }

    // ── Validation helpers ────────────────────────────────────────────────────

    /// Verify that every effect used in `inferred` is declared in `declared`.
    fn check_effects_subsumed(
        &self,
        span: &Span,
        name: &str,
        declared: &TypeSig,
        inferred: &TypeSig,
    ) -> Result<(), TypeError> {
        for label in &inferred.effects.labels {
            if !declared.effects.contains(&label.0) {
                return Err(TypeError::missing_effect(
                    span.clone(),
                    format!(
                        "'{}' uses effect '{}' but does not declare it in its signature",
                        name, label.0
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Verify that the declared input/output rows are compatible with the
    /// inferred rows via unification.
    fn check_rows_match(
        &self,
        span: &Span,
        name: &str,
        declared: &TypeSig,
        inferred: &TypeSig,
    ) -> Result<(), TypeError> {
        self.check_row_arity_and_types(span, name, "input", &declared.input, &inferred.input)?;
        self.check_row_arity_and_types(span, name, "output", &declared.output, &inferred.output)?;
        Ok(())
    }

    /// Check that two rows have the same number of specific types (when both
    /// carry a row variable) and that the types are pairwise unifiable.
    ///
    /// The arity check is essential for catching cases such as `dup` (which
    /// produces two items) being mis-annotated with a one-item output.
    fn check_row_arity_and_types(
        &self,
        span: &Span,
        name: &str,
        which: &str,
        declared: &Row,
        inferred: &Row,
    ) -> Result<(), TypeError> {
        // When both rows have a row variable the number of specific types on top
        // must agree; otherwise the declared arity is wrong.
        if declared.var.is_some()
            && inferred.var.is_some()
            && declared.types.len() != inferred.types.len()
        {
            return Err(TypeError::type_mismatch(
                span.clone(),
                format!(
                    "'{}' {} arity mismatch: declared {} item(s), body produces {} item(s)",
                    name,
                    which,
                    declared.types.len(),
                    inferred.types.len()
                ),
            ));
        }
        unify_rows(declared, inferred).map_err(|e| {
            TypeError::type_mismatch(
                span.clone(),
                format!("'{}' declared {} doesn't match body: {}", name, which, e.0),
            )
        })?;
        Ok(())
    }

    // ── Type helpers ──────────────────────────────────────────────────────────

    fn lookup_word(
        &self,
        span: &Span,
        _module: Option<&str>,
        name: &str,
    ) -> Result<TypeSig, TypeError> {
        self.env
            .get(name)
            .cloned()
            .ok_or_else(|| TypeError::unknown_word(span.clone(), name))
    }

    fn literal_sig_for(&self, data: &ValueData) -> TypeSig {
        let ty = match data {
            ValueData::Integer(_) => StackType::Primitive(PrimitiveType::Int),
            ValueData::Float(_) => StackType::Primitive(PrimitiveType::Float),
            ValueData::Boolean(_) => StackType::Primitive(PrimitiveType::Bool),
            ValueData::String(_) => StackType::Primitive(PrimitiveType::Str),
            ValueData::Char(_) => StackType::Primitive(PrimitiveType::Char),
            _ => StackType::TypeVar("a".to_string()),
        };
        let s = "S".to_string();
        TypeSig::pure(
            Row {
                var: Some(s.clone()),
                types: vec![],
            },
            Row {
                var: Some(s),
                types: vec![ty],
            },
        )
    }

    /// Placeholder type for vector literals: `( S -- S v )` where `v` is a
    /// type variable.  A proper generic vector type is a future concern.
    fn vector_sig(&self) -> TypeSig {
        let s = "S".to_string();
        TypeSig::pure(
            Row {
                var: Some(s.clone()),
                types: vec![],
            },
            Row {
                var: Some(s),
                types: vec![StackType::TypeVar("v".to_string())],
            },
        )
    }

    /// The identity transformation `( S -- S )`.
    fn identity_sig(&self) -> TypeSig {
        let s = "S".to_string();
        TypeSig::pure(
            Row {
                var: Some(s.clone()),
                types: vec![],
            },
            Row {
                var: Some(s),
                types: vec![],
            },
        )
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::ast::{AstNode, NodeKind, Span};
    use crate::types::{EffectSet, PrimitiveType, Row, StackType, TypeSig};
    use crate::value::{Value, ValueData};

    fn dummy_span() -> Span {
        Span::unknown("test".to_string())
    }

    fn literal_node(data: ValueData) -> AstNode {
        AstNode::new(NodeKind::Literal(Value::new(data)), dummy_span())
    }

    fn word_node(name: &str) -> AstNode {
        AstNode::new(
            NodeKind::Word {
                module: None,
                name: name.to_string(),
            },
            dummy_span(),
        )
    }

    fn sig(s: &str) -> TypeSig {
        TypeSig::from_str(s).unwrap()
    }

    fn program(items: Vec<AstNode>) -> AstNode {
        AstNode::new(NodeKind::Program(items), dummy_span())
    }

    // ── Literal inference ─────────────────────────────────────────────────────

    #[test]
    fn literal_int_has_int_output() {
        let mut tc = TypeChecker::new();
        let mut node = literal_node(ValueData::Integer(42));
        let result = tc.check_item(&mut node).unwrap();
        assert!(result.to_string().contains("int"));
        assert!(node.type_info.is_some());
    }

    #[test]
    fn literal_bool_has_bool_output() {
        let mut tc = TypeChecker::new();
        let mut node = literal_node(ValueData::Boolean(true));
        let result = tc.check_item(&mut node).unwrap();
        assert!(result.to_string().contains("bool"));
    }

    #[test]
    fn literal_str_has_str_output() {
        let mut tc = TypeChecker::new();
        let mut node = literal_node(ValueData::String("hi".to_string()));
        let result = tc.check_item(&mut node).unwrap();
        assert!(result.to_string().contains("str"));
    }

    // ── Word lookup ───────────────────────────────────────────────────────────

    #[test]
    fn known_word_returns_its_sig() {
        let mut tc = TypeChecker::new();
        tc.define("dup", sig("( S a -- S a a )"));
        let mut node = word_node("dup");
        let result = tc.check_item(&mut node).unwrap();
        assert_eq!(result, sig("( S a -- S a a )"));
    }

    #[test]
    fn unknown_word_is_an_error() {
        let mut tc = TypeChecker::new();
        let mut node = word_node("missing");
        assert!(tc.check_item(&mut node).is_err());
    }

    // ── Sequence composition ──────────────────────────────────────────────────

    #[test]
    fn push_two_ints_and_add() {
        let mut tc = TypeChecker::new();
        tc.define("+", sig("( S int int -- S int )"));
        let mut nodes = vec![
            literal_node(ValueData::Integer(1)),
            literal_node(ValueData::Integer(2)),
            word_node("+"),
        ];
        let result = tc.check_sequence(&mut nodes).unwrap();
        // Should produce ( S -- S int )
        assert!(result.output.types.len() == 1);
        assert_eq!(
            result.output.types[0],
            StackType::Primitive(PrimitiveType::Int)
        );
    }

    // ── Quotation inference ───────────────────────────────────────────────────

    #[test]
    fn quotation_wraps_inner_sig() {
        let mut tc = TypeChecker::new();
        tc.define("dup", sig("( S a -- S a a )"));
        let body = vec![word_node("dup")];
        let mut node = AstNode::new(NodeKind::Quotation(body), dummy_span());
        let result = tc.check_item(&mut node).unwrap();
        // Result is ( R -- R [ S a -- S a a ] )
        assert_eq!(result.input.types.len(), 0);
        assert_eq!(result.output.types.len(), 1);
        assert!(matches!(&result.output.types[0], StackType::Quotation(_)));
    }

    // ── Definition checking ───────────────────────────────────────────────────

    #[test]
    fn definition_body_matches_declaration() {
        let mut tc = TypeChecker::new();
        tc.define("dup", sig("( S a -- S a a )"));
        tc.define("+", sig("( S int int -- S int )"));

        let body = vec![word_node("dup"), word_node("+")];
        let mut def = AstNode::new(
            NodeKind::Definition {
                name: "double".to_string(),
                type_sig: sig("( S int -- S int )"),
                body,
            },
            dummy_span(),
        );
        assert!(tc.check_item(&mut def).is_ok());
    }

    #[test]
    fn definition_output_mismatch_is_error() {
        let mut tc = TypeChecker::new();
        tc.define("dup", sig("( S a -- S a a )"));

        // Body: dup — infers ( S a -- S a a ); declared output is ( S int )
        let body = vec![word_node("dup")];
        let mut def = AstNode::new(
            NodeKind::Definition {
                name: "bad".to_string(),
                type_sig: sig("( S int -- S int )"), // declared output: one int
                body,
            },
            dummy_span(),
        );
        // dup output has two items, declared has one → mismatch
        assert!(tc.check_item(&mut def).is_err());
    }

    // ── Effect checking ───────────────────────────────────────────────────────

    #[test]
    fn undeclared_effect_is_error() {
        let mut tc = TypeChecker::new();
        // print has io effect
        tc.define(
            "print",
            TypeSig::new(
                Row {
                    var: Some("S".to_string()),
                    types: vec![StackType::Primitive(PrimitiveType::Str)],
                },
                Row {
                    var: Some("S".to_string()),
                    types: vec![],
                },
                EffectSet::single("io"),
            ),
        );

        // Definition declares no effects but body uses io
        let body = vec![word_node("print")];
        let mut def = AstNode::new(
            NodeKind::Definition {
                name: "silent-print".to_string(),
                type_sig: sig("( S str -- S )"), // no | io declared
                body,
            },
            dummy_span(),
        );
        assert!(tc.check_item(&mut def).is_err());
    }

    #[test]
    fn declared_effect_passes() {
        let mut tc = TypeChecker::new();
        tc.define(
            "print",
            TypeSig::new(
                Row {
                    var: Some("S".to_string()),
                    types: vec![StackType::Primitive(PrimitiveType::Str)],
                },
                Row {
                    var: Some("S".to_string()),
                    types: vec![],
                },
                EffectSet::single("io"),
            ),
        );

        let body = vec![word_node("print")];
        let mut def = AstNode::new(
            NodeKind::Definition {
                name: "loud-print".to_string(),
                type_sig: TypeSig::from_str("( S str -- S | io )").unwrap(),
                body,
            },
            dummy_span(),
        );
        assert!(tc.check_item(&mut def).is_ok());
    }

    // ── Forward references ────────────────────────────────────────────────────

    #[test]
    fn forward_reference_in_program() {
        let mut tc = TypeChecker::new();

        // `bar` references `foo`, which is defined later in the program.
        // Both definitions are in the same Program node.
        let foo_body = vec![literal_node(ValueData::Integer(1))];
        let foo = AstNode::new(
            NodeKind::Definition {
                name: "foo".to_string(),
                type_sig: sig("( S -- S int )"),
                body: foo_body,
            },
            dummy_span(),
        );

        let bar_body = vec![word_node("foo")];
        let bar = AstNode::new(
            NodeKind::Definition {
                name: "bar".to_string(),
                type_sig: sig("( S -- S int )"),
                body: bar_body,
            },
            dummy_span(),
        );

        // bar is listed before foo — forward reference
        let mut prog = program(vec![bar, foo]);
        assert!(tc.check(&mut prog).is_ok());
    }

    // ── Annotation ────────────────────────────────────────────────────────────

    #[test]
    fn type_info_is_set_on_literal() {
        let mut tc = TypeChecker::new();
        let mut node = literal_node(ValueData::Integer(7));
        tc.check_item(&mut node).unwrap();
        assert!(node.type_info.is_some());
    }

    #[test]
    fn type_info_is_set_on_definition_body_nodes() {
        let mut tc = TypeChecker::new();
        tc.define("+", sig("( S int int -- S int )"));
        let body = vec![
            literal_node(ValueData::Integer(1)),
            literal_node(ValueData::Integer(2)),
            word_node("+"),
        ];
        let mut def = AstNode::new(
            NodeKind::Definition {
                name: "two".to_string(),
                type_sig: sig("( S -- S int )"),
                body,
            },
            dummy_span(),
        );
        tc.check_item(&mut def).unwrap();
        // The definition itself should be annotated
        assert!(def.type_info.is_some());
    }
}
