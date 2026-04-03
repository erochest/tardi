## ADDED Requirements

### Requirement: Effect variables are named in the pipe position of type signatures
The type system SHALL allow lowercase identifiers that do not match a known effect label to appear in the `| ...` position of a type signature, where they act as effect variables ranging over unknown effect sets. An effect variable represents "whatever effects the caller provides" and is resolved at each call site.

#### Scenario: Single effect variable in output position
- **WHEN** a type signature `( S [ S -- T | e ] -- T | e )` is parsed
- **THEN** the outer effect row contains one variable `e` and no concrete labels

#### Scenario: Union of two effect variables
- **WHEN** a type signature `( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )` is parsed
- **THEN** the outer effect row contains two variables `e` and `f` and no concrete labels

#### Scenario: Effect variable mixed with concrete label
- **WHEN** a type signature `( S [ S -- T | e ] -- T | io e )` is parsed
- **THEN** the outer effect row contains concrete label `io` and variable `e`

#### Scenario: Known label is not treated as a variable
- **WHEN** the identifiers `io`, `alloc`, `error`, `rand`, or `time` appear in the `| ...` position
- **THEN** they are treated as concrete effect labels, not as effect variables

### Requirement: Effect variables are bound at call sites
When a word with an effect variable in its signature is called, the type checker SHALL bind each effect variable to the concrete effect set contributed at that position by the call's arguments. After binding, the outer call site's declared effects must be a superset of the union of all bound variable values.

#### Scenario: Variable bound from quotation argument
- **WHEN** `apply` typed `( S [ S -- T | e ] -- T | e )` is called with a quotation whose inferred effects are `{ io }`
- **THEN** `e` is bound to `{ io }` and the call site must declare at least `io`

#### Scenario: Two variables bound independently
- **WHEN** `if` typed `( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )` is called with branch quotations having effects `{ io }` and `∅`
- **THEN** `e` is bound to `{ io }`, `f` is bound to `∅`, and the call site must declare at least `{ io }`

#### Scenario: Variable bound to empty set propagates nothing
- **WHEN** an effect variable is bound to an empty effect set (pure quotation)
- **THEN** the call site inherits no effects from that variable

#### Scenario: Consistent re-use of the same variable
- **WHEN** the same variable name `e` appears in both the quotation parameter and the output effect row
- **THEN** the type checker uses a single binding for `e` across both positions

### Requirement: Effect variable unification is local to each call site
Effect variable bindings SHALL be fresh for each call site. Variables with the same name in different call sites do not share a binding. The substitution map is not persisted across word calls.

#### Scenario: Same variable name reused across calls
- **WHEN** two different calls to `apply` appear in a word body, each with quotations of different effects
- **THEN** the `e` variable is independently bound at each call, and the word's total effect set is the union of both bound values

### Requirement: Effect variables round-trip through display
When a `TypeSig` containing effect variables is converted to a string and parsed again, the result SHALL equal the original.

#### Scenario: Single variable round-trips
- **WHEN** `( S [ S -- T | e ] -- T | e ).to_string()` is parsed as a `TypeSig`
- **THEN** the result equals the original `TypeSig` with variable `e` in the effect row

#### Scenario: Variable plus concrete label round-trips
- **WHEN** `( S [ S -- T | e ] -- T | io e ).to_string()` is parsed
- **THEN** the result has concrete label `io` and variable `e` in the outer effect row
