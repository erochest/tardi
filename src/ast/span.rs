use crate::value::Pos;

/// Identifies a source file or input unit.
pub type SourceId = String;

/// A single position (line/column/byte offset) in source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    /// 1-based line number
    pub line: usize,
    /// 1-based column number
    pub column: usize,
    /// 0-based byte offset from start of source
    pub offset: usize,
}

/// A source range from `start` (inclusive) to `end` (exclusive).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    pub source: SourceId,
    pub start: Position,
    pub end: Position,
}

impl Span {
    /// Build a `Span` from a scanner `Pos` (single-token range).
    pub fn from_pos(source: SourceId, pos: &Pos) -> Self {
        Span {
            start: Position {
                line: pos.line,
                column: pos.column,
                offset: pos.offset,
            },
            end: Position {
                line: pos.line,
                column: pos.column + pos.length,
                offset: pos.offset + pos.length,
            },
            source,
        }
    }

    /// Merge two spans into one covering from `start.start` to `end.end`.
    /// Panics in debug builds if the spans are from different sources.
    pub fn merge(start: &Span, end: &Span) -> Span {
        debug_assert_eq!(
            start.source, end.source,
            "cannot merge spans from different sources"
        );
        Span {
            source: start.source.clone(),
            start: start.start.clone(),
            end: end.end.clone(),
        }
    }

    /// A sentinel span used when source location is unavailable.
    pub fn unknown(source: SourceId) -> Self {
        let pos = Position { line: 0, column: 0, offset: 0 };
        Span { source, start: pos.clone(), end: pos }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.source, self.start.line, self.start.column
        )
    }
}
