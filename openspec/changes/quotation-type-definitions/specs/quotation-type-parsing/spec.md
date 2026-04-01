## ADDED Requirements

### Requirement: Quotation types are parseable in type signature strings
The `TypeSig` parser SHALL accept `[ inputs -- outputs ]` and `[ inputs -- outputs | effects ]` spans as stack items in either the input or output row of a type signature, producing a `StackType::Quotation` value. Nesting SHALL be supported to arbitrary depth.

#### Scenario: Simple quotation type in input row
- **WHEN** the string `"( S vec [ x -- y ] -- S vec )"` is parsed as a `TypeSig`
- **THEN** the input row contains two items: a named type `vec` and a `StackType::Quotation` with inner sig `( x -- y )`

#### Scenario: Quotation type with row variable
- **WHEN** the string `"( S a [ S a -- S a ] -- S a a )"` is parsed
- **THEN** the inner quotation sig has `var = Some("S")` on both its input and output rows

#### Scenario: Quotation type with effects
- **WHEN** the string `"( S vec [ x -- y | io ] -- S vec | io )"` is parsed
- **THEN** the inner quotation sig has effect set `{io}` and the outer sig also has effect set `{io}`

#### Scenario: Nested quotation types
- **WHEN** the string `"( S [ [ x -- y ] -- z ] -- S z )"` is parsed
- **THEN** the input quotation type contains another quotation type as its input item

#### Scenario: Outer `--` not confused with inner `--`
- **WHEN** a type signature contains one or more `--` inside `[ ... ]` brackets
- **THEN** the parser finds the correct outer `--` boundary (depth 0) and does not mis-split on inner ones

#### Scenario: Display round-trips through parse
- **WHEN** any `TypeSig` containing `StackType::Quotation` items is converted to a string and parsed again
- **THEN** the result equals the original `TypeSig`

### Requirement: Malformed quotation types produce parse errors
The parser SHALL reject quotation type spans that are not themselves valid type signatures.

#### Scenario: Unclosed bracket is an error
- **WHEN** the string `"( S [ x -- y -- S )"` is parsed (missing closing `]`)
- **THEN** the parser returns a `TypeSigParseError`

#### Scenario: Empty bracket span is accepted as identity type
- **WHEN** the string `"( S [ -- ] -- S )"` is parsed
- **THEN** it succeeds and the inner sig has empty input and output rows

### Requirement: Higher-order word annotations use quotation types
All built-in and bootstrap words that accept a quotation argument SHALL have their type annotations updated to use `[ ... ]` quotation types rather than a generic type variable.

#### Scenario: `dip` annotation uses quotation type
- **WHEN** the type annotation for `dip` is inspected
- **THEN** it is `( S a [ S -- S ] -- S a )` or equivalent, showing the quotation's stack contract

#### Scenario: `map` annotation uses quotation type
- **WHEN** the type annotation for `map` is inspected
- **THEN** it is `( S vec [ x -- y ] -- S vec )` or equivalent

#### Scenario: `keep` annotation uses quotation type
- **WHEN** the type annotation for `keep` is inspected
- **THEN** it is `( S a [ S a -- S b ] -- S b a )` or equivalent, showing the value is preserved
