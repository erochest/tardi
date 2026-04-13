## ADDED Requirements

### Requirement: Type signatures use row-polymorphic stack effects
The type system SHALL represent word types as stack transformations using row polymorphism. A type signature has the form `( S inputs -- S outputs )` where `S` is a row variable representing the rest of the stack beneath the named arguments.

#### Scenario: Unary word type
- **WHEN** a word consumes one value and produces one value of a different type
- **THEN** its type is representable as `( S a -- S b )` where `S` is shared across input and output

#### Scenario: Stack-preserving word type
- **WHEN** a word duplicates the top of stack
- **THEN** its type is `( S a -- S a a )`

#### Scenario: Multi-value word type
- **WHEN** a word consumes two values and produces one
- **THEN** its type is representable as `( S a b -- S c )`

### Requirement: Type signatures are written inline in word definitions
Word definitions SHALL include an inline type annotation immediately after the word name, enclosed in parentheses: `: name ( inputs -- outputs | effects ) body ;`

#### Scenario: Annotated definition accepted
- **WHEN** a definition includes a well-formed type signature
- **THEN** the parser accepts it and the type checker validates the body against the signature

#### Scenario: Missing annotation is an error
- **WHEN** a top-level definition omits the type signature
- **THEN** the type checker reports an error requiring annotation

### Requirement: Type signatures support primitive and user-defined types
The type system SHALL include primitive types (`int`, `float`, `bool`, `str`, `char`) and SHALL support user-defined named types. Type names are lowercase identifiers.

#### Scenario: Primitive type in signature
- **WHEN** a type signature references `int` or `str`
- **THEN** the type checker recognizes them as built-in primitive types

#### Scenario: Unknown type name is an error
- **WHEN** a type signature references a name that is not defined
- **THEN** the type checker reports an unknown type error with the span of the unknown name

### Requirement: Quotation types are inferred
The type checker SHALL infer the stack-effect type of quotations (`[ ... ]`) from their contents without requiring annotation.

#### Scenario: Quotation type inferred from body
- **WHEN** a quotation `[ dup + ]` appears in code
- **THEN** the type checker infers its type as `( S int int -- S int )` (assuming `+` is typed for `int`)

#### Scenario: Quotation type checked at use site
- **WHEN** a word expects a quotation of type `( S a -- S b )` and receives one of incompatible type
- **THEN** the type checker reports a type mismatch error

### Requirement: Type composition follows function composition rules
When two words are sequenced, their types compose: the output row of the first word must unify with the input row of the second.

#### Scenario: Compatible composition type-checks
- **WHEN** `dup` typed `( S a -- S a a )` is followed by `+` typed `( S int int -- S int )`
- **THEN** the sequence type-checks with unified type `( S int -- S int )`

#### Scenario: Incompatible composition is an error
- **WHEN** a word producing `str` on top of stack is followed by a word expecting `int`
- **THEN** the type checker reports a type mismatch error with spans for both words
