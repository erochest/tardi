## ADDED Requirements

### Requirement: Effects are declared in type signatures using pipe syntax
Word definitions SHALL declare their effects after a `|` separator in the type signature: `( inputs -- outputs | effect1 effect2 )`. A word with no effects omits the `|` section or writes `( inputs -- outputs )`.

#### Scenario: Single effect declared
- **WHEN** a definition is written as `: readline ( file-like -- str | io )`
- **THEN** the parser recognizes `io` as a declared effect on `readline`

#### Scenario: Multiple effects declared
- **WHEN** a definition is written as `( a -- b | io alloc )`
- **THEN** the type checker records both `io` and `alloc` in the definition's effect set

#### Scenario: No effects — both forms accepted
- **WHEN** a definition is written as `( a -- b )` without a `|` section
- **THEN** the type checker treats the effect set as empty, equivalent to `( a -- b | )`

### Requirement: Effects propagate upward via set union
If a word calls any word with effects `E`, the caller MUST declare at least those effects in its own signature. The type checker SHALL verify this by computing the union of all effects reachable from a word's body.

#### Scenario: Effect propagates to caller
- **WHEN** `readline` has effect `io` and `process` calls `readline` without declaring `io`
- **THEN** the type checker reports a missing effect error on `process`

#### Scenario: Superset of effects is valid
- **WHEN** a word declares `| io alloc` but only calls words with `| io`
- **THEN** the type checker accepts the definition (declaring more effects than needed is allowed)

#### Scenario: Exact match is valid
- **WHEN** a word declares exactly the effects that its body requires
- **THEN** the type checker accepts the definition

### Requirement: Effects compose across sequenced words
When two words are sequenced, their effect sets are unioned to form the effect set of the combined sequence.

#### Scenario: Sequential effects union
- **WHEN** word `A` has effects `{ io }` and word `B` has effects `{ alloc }` and they are sequenced
- **THEN** the composed sequence has effects `{ io, alloc }`

#### Scenario: Repeated effects deduplicated
- **WHEN** two words both have effect `io` and are sequenced
- **THEN** the composed effect set contains `io` exactly once

### Requirement: Effect labels are lowercase identifiers
Effect labels SHALL be lowercase identifiers. The type system includes the following built-in effect labels: `io`, `alloc`, `error`, `rand`, `time`. User code MAY define additional effect labels (mechanism TBD in the algebraic-effects change).

#### Scenario: Built-in effect label recognized
- **WHEN** a type signature includes `| io`
- **THEN** the type checker recognizes `io` as the built-in I/O effect

#### Scenario: Unknown effect label is an error
- **WHEN** a type signature includes an effect label not in scope
- **THEN** the type checker reports an unknown effect error with the span of the label

### Requirement: Effect mismatch errors include spans and labels
When the type checker detects a missing or unexpected effect, it SHALL report an error identifying the missing effect label, the word where it was detected, and the word that introduced it.

#### Scenario: Error identifies missing effect and source
- **WHEN** `process` calls `readline` (which has `| io`) without declaring `| io`
- **THEN** the error message names `io`, identifies `process` as the caller, and identifies `readline` as the source of the effect

### Requirement: Effect system is designed for future algebraic handler extension
The effect type representation SHALL be structured so that a future `handle` combinator can consume an effect from an enclosed quotation's effect set without changing existing annotations.

#### Scenario: Effect representation is a set
- **WHEN** effects are stored internally
- **THEN** they are stored as a set (or equivalent) supporting add, union, and remove operations, so that a handler removing an effect is representable
