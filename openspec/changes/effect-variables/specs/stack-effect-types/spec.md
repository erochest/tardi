## MODIFIED Requirements

### Requirement: Type signatures use row-polymorphic stack effects
The type system SHALL represent word types as stack transformations using row polymorphism. A type signature has the form `( inputs -- outputs )` where the row variable (e.g. `S`) is optional for simple words and required when the stack context must be named explicitly (e.g. for higher-order words). Stack items may be primitive types, type variables, or quotation types written as `[ inputs -- outputs ]`. The `| ...` position accepts both concrete effect labels and effect variables (lowercase identifiers not matching any known label); the two may be mixed, e.g. `| io e`.

#### Scenario: Unary word type
- **WHEN** a word consumes one value and produces one value of a different type
- **THEN** its type is representable as `( a -- b )` (row variable omitted) or `( S a -- S b )` (row variable explicit)

#### Scenario: Stack-preserving word type
- **WHEN** a word duplicates the top of stack
- **THEN** its type is `( a -- a a )`

#### Scenario: Multi-value word type
- **WHEN** a word consumes two values and produces one
- **THEN** its type is representable as `( a b -- c )`

#### Scenario: Quotation type as stack item
- **WHEN** a word accepts a quotation argument
- **THEN** its type MAY name the quotation's contract explicitly using `[ ... ]` notation, e.g. `( S vec [ x -- y ] -- S vec )`

#### Scenario: Row variable required for higher-order words
- **WHEN** a word passes through stack context to a quotation it calls
- **THEN** the row variable SHALL be present to name that context, e.g. `( S a [ S a -- S b ] -- S b a )`

#### Scenario: Effect variable in type signature
- **WHEN** a type signature contains an unknown lowercase identifier in the `| ...` position, such as `( S [ S -- T | e ] -- T | e )`
- **THEN** the parser produces a `TypeSig` whose outer effect row has variable `e` and no concrete labels

#### Scenario: Effect variable mixed with concrete label
- **WHEN** a type signature is written as `( S [ S -- T | e ] -- T | io e )`
- **THEN** the parser produces a `TypeSig` whose outer effect row has concrete label `io` and variable `e`

## ADDED Requirements

### Requirement: Row variable is optional in type signatures
The row variable SHALL be omissible from a type signature when it carries no additional information. A signature without a row variable (e.g. `( int int -- int )`) is compositionally equivalent to one with an implicit shared row variable (e.g. `( S int int -- S int )`). The parser and type checker SHALL treat absent row variables as implicit.

#### Scenario: Short-form signature parses correctly
- **WHEN** the string `"( int int -- int )"` is parsed
- **THEN** both input and output rows have `var = None` and the appropriate primitive types

#### Scenario: Short-form composes with explicit-row-var signature
- **WHEN** `( a -- a a )` is composed with `( S a -- S )`
- **THEN** the result is a valid composed type with no error

#### Scenario: Short-form round-trips through display
- **WHEN** a `TypeSig` with `var = None` rows is converted to a string
- **THEN** the string does not contain a row variable prefix
