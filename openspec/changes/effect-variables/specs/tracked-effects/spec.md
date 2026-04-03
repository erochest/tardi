## MODIFIED Requirements

### Requirement: Effects propagate upward via set union
If a word calls any word with effects `E`, the caller MUST declare at least those effects in its own signature. The type checker SHALL verify this by computing the union of all effects reachable from a word's body. When a called word has effect variables, those variables are bound to the concrete effect sets contributed by the call's quotation arguments, and the resolved concrete set is added to the caller's required effects.

#### Scenario: Effect propagates to caller
- **WHEN** `readline` has effect `io` and `process` calls `readline` without declaring `io`
- **THEN** the type checker reports a missing effect error on `process`

#### Scenario: Superset of effects is valid
- **WHEN** a word declares `| io alloc` but only calls words with `| io`
- **THEN** the type checker accepts the definition (declaring more effects than needed is allowed)

#### Scenario: Exact match is valid
- **WHEN** a word declares exactly the effects that its body requires
- **THEN** the type checker accepts the definition

#### Scenario: Effect variable resolved and propagated
- **WHEN** a word calls `apply` typed `( S [ S -- T | e ] -- T | e )` with a quotation whose effects are `{ io }`
- **THEN** `e` resolves to `{ io }` and the calling word must declare at least `io`

#### Scenario: Effect variable resolving to empty does not propagate
- **WHEN** a word calls `apply` with a pure (no-effect) quotation
- **THEN** `e` resolves to `∅` and the calling word inherits no effects from that call

### Requirement: Effects compose across sequenced words
When two words are sequenced, their effect sets are unioned to form the effect set of the combined sequence. Effect variable bindings from each call are resolved before union.

#### Scenario: Sequential effects union
- **WHEN** word `A` has effects `{ io }` and word `B` has effects `{ alloc }` and they are sequenced
- **THEN** the composed sequence has effects `{ io, alloc }`

#### Scenario: Repeated effects deduplicated
- **WHEN** two words both have effect `io` and are sequenced
- **THEN** the composed effect set contains `io` exactly once

#### Scenario: Two apply calls with different quotation effects compose
- **WHEN** a word calls `apply` twice — first with a `{ io }` quotation, then with an `{ alloc }` quotation
- **THEN** the word's total required effects are `{ io, alloc }` and it must declare both
