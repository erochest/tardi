pub mod span;

pub use span::{Position, SourceId, Span};

use crate::types::TypeSig;
use crate::value::Value;

/// Type information attached to an AST node by the type checker.
/// This is a placeholder; the full representation is implemented in task group 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    /// The inferred/checked type signature of this node.
    pub type_sig: TypeSig,
}

/// The syntactic kind of an AST node.
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// A literal value: integer, float, boolean, string, or character.
    Literal(Value),

    /// A reference to a word, optionally module-qualified.
    Word {
        module: Option<String>,
        name: String,
    },

    /// An anonymous quotation `[ ... ]`.
    Quotation(Vec<AstNode>),

    /// A vector/list literal `{ ... }`.
    /// Items are parsed as AST nodes (may include quotations, words, etc.).
    Vector(Vec<AstNode>),

    /// A named word definition `: name ( sig ) body ;`.
    Definition {
        name: String,
        type_sig: TypeSig,
        body: Vec<AstNode>,
    },

    /// A macro definition `MACRO: name body ;`.
    /// The body is kept as raw tokens for the compiler to compile and register.
    MacroDefinition {
        name: String,
        body: Vec<Value>,
    },

    /// The top-level sequence of nodes in a source unit.
    Program(Vec<AstNode>),
}

/// A node in the Tardi AST, carrying its kind, source span, and optional type info.
#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: NodeKind,
    pub span: Span,
    pub type_info: Option<TypeInfo>,
}

impl AstNode {
    pub fn new(kind: NodeKind, span: Span) -> Self {
        AstNode { kind, span, type_info: None }
    }

    /// Returns direct child nodes (one level deep).
    pub fn children(&self) -> Vec<&AstNode> {
        match &self.kind {
            NodeKind::Literal(_)
            | NodeKind::Word { .. }
            | NodeKind::MacroDefinition { .. } => vec![],
            NodeKind::Quotation(nodes) => nodes.iter().collect(),
            NodeKind::Vector(nodes) => nodes.iter().collect(),
            NodeKind::Definition { body, .. } => body.iter().collect(),
            NodeKind::Program(nodes) => nodes.iter().collect(),
        }
    }

    /// Recursively visits this node and all descendants in program order,
    /// calling `f` on each node before visiting its children.
    pub fn walk<F>(&self, f: &mut F)
    where
        F: FnMut(&AstNode),
    {
        f(self);
        for child in self.children() {
            child.walk(f);
        }
    }

    /// Fold over this node and all descendants in program order.
    /// `f` receives the accumulator and current node; returns the new accumulator.
    pub fn fold<A, F>(&self, init: A, f: &mut F) -> A
    where
        F: FnMut(A, &AstNode) -> A,
    {
        let acc = f(init, self);
        self.children().iter().fold(acc, |a, child| child.fold(a, f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Value, ValueData};

    fn dummy_span() -> Span {
        Span::unknown("test".to_string())
    }

    fn literal_node(n: i64) -> AstNode {
        AstNode::new(
            NodeKind::Literal(Value::new(ValueData::Integer(n))),
            dummy_span(),
        )
    }

    fn word_node(name: &str) -> AstNode {
        AstNode::new(
            NodeKind::Word { module: None, name: name.to_string() },
            dummy_span(),
        )
    }

    #[test]
    fn literal_has_no_children() {
        let node = literal_node(42);
        assert!(node.children().is_empty());
    }

    #[test]
    fn word_has_no_children() {
        let node = word_node("dup");
        assert!(node.children().is_empty());
    }

    #[test]
    fn quotation_children() {
        let inner = vec![literal_node(1), word_node("dup")];
        let node = AstNode::new(NodeKind::Quotation(inner), dummy_span());
        assert_eq!(node.children().len(), 2);
    }

    #[test]
    fn program_children() {
        let nodes = vec![literal_node(1), literal_node(2), word_node("+")];
        let prog = AstNode::new(NodeKind::Program(nodes), dummy_span());
        assert_eq!(prog.children().len(), 3);
    }

    #[test]
    fn walk_visits_all_nodes() {
        // Program [ [ 1 ] + ]
        let inner = AstNode::new(
            NodeKind::Quotation(vec![literal_node(1)]),
            dummy_span(),
        );
        let prog = AstNode::new(
            NodeKind::Program(vec![inner, word_node("+")]),
            dummy_span(),
        );

        let mut visited = 0usize;
        prog.walk(&mut |_| visited += 1);
        // program + quotation + literal(1) + word("+") = 4
        assert_eq!(visited, 4);
    }

    #[test]
    fn fold_counts_nodes() {
        let prog = AstNode::new(
            NodeKind::Program(vec![literal_node(1), literal_node(2), word_node("+")]),
            dummy_span(),
        );
        let count = prog.fold(0usize, &mut |acc, _| acc + 1);
        assert_eq!(count, 4); // program + 3 children
    }

    #[test]
    fn type_info_absent_after_construction() {
        let node = literal_node(7);
        assert!(node.type_info.is_none());
    }

    #[test]
    fn definition_has_body_as_children() {
        let body = vec![word_node("dup"), word_node("+")];
        let def = AstNode::new(
            NodeKind::Definition {
                name: "double".to_string(),
                type_sig: "( S int -- S int )".parse().unwrap(),
                body,
            },
            dummy_span(),
        );
        assert_eq!(def.children().len(), 2);
    }
}
