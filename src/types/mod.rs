use std::collections::BTreeSet;

// ──────────────────────────────────────────────────────────────────────────────
// Primitive types
// ──────────────────────────────────────────────────────────────────────────────

/// A concrete, named type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Int,
    Float,
    Bool,
    Str,
    Char,
    /// User-defined or opaque named type (e.g. `file-like`, `result`).
    Named(String),
}

impl PrimitiveType {
    fn from_name(s: &str) -> Self {
        match s {
            "int" => PrimitiveType::Int,
            "float" => PrimitiveType::Float,
            "bool" => PrimitiveType::Bool,
            "str" => PrimitiveType::Str,
            "char" => PrimitiveType::Char,
            other => PrimitiveType::Named(other.to_string()),
        }
    }
}

impl std::fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Int => write!(f, "int"),
            PrimitiveType::Float => write!(f, "float"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Str => write!(f, "str"),
            PrimitiveType::Char => write!(f, "char"),
            PrimitiveType::Named(n) => write!(f, "{}", n),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Stack types
// ──────────────────────────────────────────────────────────────────────────────

/// A single type that can appear on the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackType {
    /// A concrete primitive type.
    Primitive(PrimitiveType),
    /// A type variable (polymorphic), e.g. `a`, `b`.
    TypeVar(String),
    /// An anonymous quotation type `[ sig ]`.
    Quotation(Box<TypeSig>),
}

impl StackType {
    /// Parse a single stack-type token. Uppercase = type variable by convention;
    /// lowercase single-char = type variable; known names = primitives.
    fn from_token(s: &str) -> Self {
        // Single lowercase letter → type variable
        if s.len() == 1 && s.chars().all(|c| c.is_ascii_lowercase()) {
            return StackType::TypeVar(s.to_string());
        }
        // Lowercase multi-char that isn't a primitive → type var or named type
        // We treat it as a named primitive (user-defined type).
        StackType::Primitive(PrimitiveType::from_name(s))
    }
}

impl std::fmt::Display for StackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StackType::Primitive(p) => write!(f, "{}", p),
            StackType::TypeVar(v) => write!(f, "{}", v),
            StackType::Quotation(sig) => write!(f, "[ {} ]", sig),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Rows
// ──────────────────────────────────────────────────────────────────────────────

/// A stack row: an optional row variable followed by zero or more stack types.
///
/// Represents the portion of a stack effect on one side of `--`:
///   `S a b`  →  var=Some("S"), types=[a, b]
///   `int`    →  var=None, types=[int]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Row variable (convention: uppercase), if present.
    pub var: Option<String>,
    /// Types above the row variable, from bottom to top.
    pub types: Vec<StackType>,
}

impl Row {
    pub fn empty() -> Self {
        Row { var: None, types: vec![] }
    }

    /// Parse a space-separated token list into a `Row`.
    /// The first token is the row variable if it starts with an uppercase letter.
    fn from_tokens(tokens: &[&str]) -> Self {
        let mut iter = tokens.iter().peekable();
        let var = match iter.peek() {
            Some(t) if t.starts_with(|c: char| c.is_ascii_uppercase()) => {
                let v = iter.next().unwrap();
                Some((*v).to_string())
            }
            _ => None,
        };
        let types = iter.map(|t| StackType::from_token(t)).collect();
        Row { var, types }
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = &self.var {
            write!(f, "{}", v)?;
            if !self.types.is_empty() {
                write!(f, " ")?;
            }
        }
        let parts: Vec<String> = self.types.iter().map(|t| t.to_string()).collect();
        write!(f, "{}", parts.join(" "))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Effect labels and sets
// ──────────────────────────────────────────────────────────────────────────────

/// A single effect label (lowercase identifier).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectLabel(pub String);

impl EffectLabel {
    pub fn new(label: impl Into<String>) -> Self {
        EffectLabel(label.into())
    }
}

impl std::fmt::Display for EffectLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A set of effect labels.  Ordered for deterministic display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSet {
    pub labels: BTreeSet<EffectLabel>,
}

impl EffectSet {
    pub fn empty() -> Self {
        EffectSet { labels: BTreeSet::new() }
    }

    pub fn single(label: impl Into<String>) -> Self {
        let mut s = EffectSet::empty();
        s.labels.insert(EffectLabel::new(label));
        s
    }

    pub fn from_labels(labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        EffectSet {
            labels: labels.into_iter().map(|l| EffectLabel::new(l)).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Set union: returns a new `EffectSet` containing all labels from both sets.
    pub fn union(&self, other: &EffectSet) -> EffectSet {
        EffectSet {
            labels: self.labels.union(&other.labels).cloned().collect(),
        }
    }

    pub fn contains(&self, label: &str) -> bool {
        self.labels.contains(&EffectLabel(label.to_string()))
    }
}

impl std::fmt::Display for EffectSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self.labels.iter().map(|l| l.to_string()).collect();
        write!(f, "{}", parts.join(" "))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Type signatures
// ──────────────────────────────────────────────────────────────────────────────

/// A complete word type: input row, output row, and effect set.
///
/// Syntax: `( S a b -- S c | io alloc )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSig {
    pub input: Row,
    pub output: Row,
    pub effects: EffectSet,
}

impl TypeSig {
    pub fn new(input: Row, output: Row, effects: EffectSet) -> Self {
        TypeSig { input, output, effects }
    }

    pub fn pure(input: Row, output: Row) -> Self {
        TypeSig { input, output, effects: EffectSet::empty() }
    }

    /// Compose two type signatures in sequence (self followed by next).
    ///
    /// Requires that `self.output` unifies with `next.input`.
    /// Returns an error describing the mismatch if they don't.
    pub fn compose(&self, next: &TypeSig) -> Result<TypeSig, UnificationError> {
        // Verify that the stacks are compatible at the join point.
        unify_rows(&self.output, &next.input)?;

        // Compute the "remaining base": the part of self.output that next.input
        // does NOT consume.  next.input.types[..b_len] are consumed; any types in
        // self.output that appear *below* them (i.e., the first a_len - b_len) are
        // still on the stack when next finishes.
        let a_len = self.output.types.len();
        let b_len = next.input.types.len();
        let remaining_base = Row {
            var: self.output.var.clone(),
            types: if a_len > b_len {
                self.output.types[..a_len - b_len].to_vec()
            } else {
                vec![]
            },
        };

        // Substitute next.input's row variable in next.output with the remaining
        // base, giving the composed output row.
        let composed_output = substitute_row(&next.output, &next.input.var, &remaining_base);
        let effects = self.effects.union(&next.effects);
        Ok(TypeSig::new(self.input.clone(), composed_output, effects))
    }
}

impl std::fmt::Display for TypeSig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "( {} -- {}", self.input, self.output)?;
        if !self.effects.is_empty() {
            write!(f, " | {}", self.effects)?;
        }
        write!(f, " )")
    }
}

impl std::str::FromStr for TypeSig {
    type Err = TypeSigParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_type_sig(s)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Parsing
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSigParseError(pub String);

impl std::fmt::Display for TypeSigParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type signature parse error: {}", self.0)
    }
}

fn parse_type_sig(s: &str) -> Result<TypeSig, TypeSigParseError> {
    let s = s.trim();
    let s = s
        .strip_prefix('(')
        .ok_or_else(|| TypeSigParseError(format!("expected '(' at start, got: {:?}", s)))?;
    let s = s
        .strip_suffix(')')
        .ok_or_else(|| TypeSigParseError("expected ')' at end".to_string()))?;
    let s = s.trim();

    // Split on `--`
    let (input_str, rest) = s
        .split_once("--")
        .ok_or_else(|| TypeSigParseError("expected '--' in type signature".to_string()))?;

    // Split rest on `|` for effects
    let (output_str, effects_str) = match rest.split_once('|') {
        Some((o, e)) => (o, Some(e)),
        None => (rest, None),
    };

    let input_tokens: Vec<&str> = input_str.split_whitespace().collect();
    let output_tokens: Vec<&str> = output_str.split_whitespace().collect();

    let input = Row::from_tokens(&input_tokens);
    let output = Row::from_tokens(&output_tokens);
    let effects = match effects_str {
        None => EffectSet::empty(),
        Some(e) => EffectSet::from_labels(e.split_whitespace()),
    };

    Ok(TypeSig { input, output, effects })
}

// ──────────────────────────────────────────────────────────────────────────────
// Unification
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnificationError(pub String);

impl std::fmt::Display for UnificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type unification error: {}", self.0)
    }
}

/// Unify two rows.  Returns the "unified" row if they are compatible.
///
/// Two rows are compatible when:
/// - One or both have a row variable (polymorphic) that can absorb the difference, OR
/// - Both have the same concrete types in the same order.
///
/// This is a simplified first-order unification sufficient for checking annotated
/// definitions.  Full HM inference is deferred.
pub fn unify_rows(a: &Row, b: &Row) -> Result<Row, UnificationError> {
    // Check that concrete type lists match at the tops
    let a_len = a.types.len();
    let b_len = b.types.len();

    if a_len == b_len {
        // Same length: types must match pairwise
        for (ta, tb) in a.types.iter().zip(b.types.iter()) {
            unify_stack_types(ta, tb)?;
        }
        // Row vars must be compatible
        let var = unify_row_vars(a.var.as_deref(), b.var.as_deref())?;
        Ok(Row { var, types: a.types.clone() })
    } else if a_len > b_len {
        // `a` has more concrete types than `b` — b's row var absorbs the difference
        if b.var.is_none() {
            return Err(UnificationError(format!(
                "rows have incompatible lengths ({} vs {}) and no row variable to absorb",
                a_len, b_len
            )));
        }
        let suffix = &a.types[a_len - b_len..];
        for (ta, tb) in suffix.iter().zip(b.types.iter()) {
            unify_stack_types(ta, tb)?;
        }
        Ok(Row { var: a.var.clone(), types: a.types.clone() })
    } else {
        // `b` has more concrete types — a's row var absorbs the difference
        if a.var.is_none() {
            return Err(UnificationError(format!(
                "rows have incompatible lengths ({} vs {}) and no row variable to absorb",
                a_len, b_len
            )));
        }
        let suffix = &b.types[b_len - a_len..];
        for (ta, tb) in a.types.iter().zip(suffix.iter()) {
            unify_stack_types(ta, tb)?;
        }
        Ok(Row { var: b.var.clone(), types: b.types.clone() })
    }
}

fn unify_stack_types(a: &StackType, b: &StackType) -> Result<(), UnificationError> {
    match (a, b) {
        // Type variables unify with anything
        (StackType::TypeVar(_), _) | (_, StackType::TypeVar(_)) => Ok(()),
        (StackType::Primitive(pa), StackType::Primitive(pb)) => {
            if pa == pb {
                Ok(())
            } else {
                Err(UnificationError(format!(
                    "type mismatch: {} vs {}",
                    pa, pb
                )))
            }
        }
        (StackType::Quotation(sa), StackType::Quotation(sb)) => {
            // Quotation types unify if their sigs unify
            unify_rows(&sa.input, &sb.input)?;
            unify_rows(&sa.output, &sb.output)?;
            Ok(())
        }
        _ => Err(UnificationError(format!("type mismatch: {:?} vs {:?}", a, b))),
    }
}

fn unify_row_vars(a: Option<&str>, b: Option<&str>) -> Result<Option<String>, UnificationError> {
    match (a, b) {
        (None, None) => Ok(None),
        (Some(v), None) | (None, Some(v)) => Ok(Some(v.to_string())),
        (Some(va), Some(vb)) => {
            if va == vb {
                Ok(Some(va.to_string()))
            } else {
                // Different row variable names — still compatible (we just pick one)
                Ok(Some(va.to_string()))
            }
        }
    }
}

/// Substitute a row variable in `row` with the row context from `context`.
///
/// Used during composition: when composing A→B with B→C, substitute B's
/// row variable in C's output with A's row variable.
fn substitute_row(row: &Row, var_name: &Option<String>, context: &Row) -> Row {
    match (&row.var, var_name) {
        (Some(rv), Some(cv)) if rv == cv => {
            // The row variable matches — substitute it with context's var
            let mut types = context.types.clone();
            types.extend(row.types.clone());
            Row { var: context.var.clone(), types }
        }
        _ => row.clone(),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── TypeSig parsing ────────────────────────────────────────────────────────

    #[test]
    fn parse_simple_sig() {
        let sig = "( S a -- S b )".parse::<TypeSig>().unwrap();
        assert_eq!(sig.input.var, Some("S".to_string()));
        assert_eq!(sig.input.types, vec![StackType::TypeVar("a".to_string())]);
        assert_eq!(sig.output.var, Some("S".to_string()));
        assert_eq!(sig.output.types, vec![StackType::TypeVar("b".to_string())]);
        assert!(sig.effects.is_empty());
    }

    #[test]
    fn parse_sig_with_effects() {
        let sig = "( S -- S str | io )".parse::<TypeSig>().unwrap();
        assert!(sig.effects.contains("io"));
        assert!(!sig.effects.contains("alloc"));
    }

    #[test]
    fn parse_sig_with_multiple_effects() {
        let sig = "( S -- S | io alloc )".parse::<TypeSig>().unwrap();
        assert!(sig.effects.contains("io"));
        assert!(sig.effects.contains("alloc"));
    }

    #[test]
    fn parse_primitive_types() {
        let sig = "( S int int -- S int )".parse::<TypeSig>().unwrap();
        assert_eq!(
            sig.input.types,
            vec![
                StackType::Primitive(PrimitiveType::Int),
                StackType::Primitive(PrimitiveType::Int),
            ]
        );
        assert_eq!(
            sig.output.types,
            vec![StackType::Primitive(PrimitiveType::Int)]
        );
    }

    #[test]
    fn parse_no_row_var() {
        let sig = "( int -- int )".parse::<TypeSig>().unwrap();
        assert_eq!(sig.input.var, None);
        assert_eq!(sig.output.var, None);
    }

    #[test]
    fn parse_error_missing_arrow() {
        let result = "( S a S b )".parse::<TypeSig>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_missing_parens() {
        let result = "S a -- S b".parse::<TypeSig>();
        assert!(result.is_err());
    }

    // ── Display ────────────────────────────────────────────────────────────────

    #[test]
    fn display_sig_no_effects() {
        let sig = "( S a -- S b )".parse::<TypeSig>().unwrap();
        assert_eq!(sig.to_string(), "( S a -- S b )");
    }

    #[test]
    fn display_sig_with_effects() {
        let sig = "( S -- S str | io )".parse::<TypeSig>().unwrap();
        assert_eq!(sig.to_string(), "( S -- S str | io )");
    }

    // ── EffectSet union ────────────────────────────────────────────────────────

    #[test]
    fn effect_union_empty() {
        let a = EffectSet::empty();
        let b = EffectSet::empty();
        assert_eq!(a.union(&b), EffectSet::empty());
    }

    #[test]
    fn effect_union_single() {
        let a = EffectSet::single("io");
        let b = EffectSet::single("alloc");
        let u = a.union(&b);
        assert!(u.contains("io"));
        assert!(u.contains("alloc"));
    }

    #[test]
    fn effect_union_deduplicates() {
        let a = EffectSet::single("io");
        let b = EffectSet::single("io");
        let u = a.union(&b);
        assert_eq!(u.labels.len(), 1);
    }

    // ── Row unification ────────────────────────────────────────────────────────

    #[test]
    fn unify_matching_rows() {
        let a = Row { var: Some("S".to_string()), types: vec![StackType::TypeVar("a".to_string())] };
        let b = Row { var: Some("S".to_string()), types: vec![StackType::TypeVar("b".to_string())] };
        assert!(unify_rows(&a, &b).is_ok());
    }

    #[test]
    fn unify_concrete_mismatch_fails() {
        let a = Row {
            var: None,
            types: vec![StackType::Primitive(PrimitiveType::Int)],
        };
        let b = Row {
            var: None,
            types: vec![StackType::Primitive(PrimitiveType::Str)],
        };
        assert!(unify_rows(&a, &b).is_err());
    }

    #[test]
    fn unify_row_absorbs_extra_types() {
        // a has more types, b has a row var to absorb the difference
        let a = Row {
            var: Some("S".to_string()),
            types: vec![
                StackType::Primitive(PrimitiveType::Int),
                StackType::Primitive(PrimitiveType::Int),
            ],
        };
        let b = Row {
            var: Some("S".to_string()),
            types: vec![StackType::Primitive(PrimitiveType::Int)],
        };
        assert!(unify_rows(&a, &b).is_ok());
    }
}
