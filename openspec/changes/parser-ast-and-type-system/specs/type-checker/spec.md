## ADDED Requirements

### Requirement: Type checker validates word definitions against their annotations
The type checker SHALL verify that the body of each word definition produces the stack effect declared in its type signature. If the inferred effect of the body does not match the declared signature, an error is reported.

#### Scenario: Correct annotation accepted
- **WHEN** the body of `: double ( S int -- S int ) dup + ;` is type-checked
- **THEN** the type checker confirms the body has type `( S int -- S int )` and accepts it

#### Scenario: Incorrect annotation rejected
- **WHEN** the body of a definition infers to `( S int -- S int int )` but the annotation says `( S int -- S int )`
- **THEN** the type checker reports a type mismatch error with the span of the definition

### Requirement: Type checker infers types for quotations
The type checker SHALL infer the stack-effect type of `[ ... ]` quotations from their contents without requiring programmer annotation.

#### Scenario: Quotation type inferred
- **WHEN** a quotation `[ 1 + ]` appears in a typed context
- **THEN** the type checker infers its type as `( S int -- S int )`

#### Scenario: Type-incorrect quotation body is an error
- **WHEN** a quotation contains a type-incompatible sequence
- **THEN** the type checker reports the error with the span of the offending node inside the quotation

### Requirement: Type checker resolves word references by name
The type checker SHALL look up each `Word` node in the current scope to find its declared type, using it to compute the type of the enclosing sequence.

#### Scenario: Known word resolved correctly
- **WHEN** a `Word("dup")` node is encountered
- **THEN** the type checker retrieves `dup`'s type `( S a -- S a a )` and uses it in composition

#### Scenario: Unknown word is an error
- **WHEN** a `Word("frobnicate")` node is encountered and no such word is defined
- **THEN** the type checker reports an undefined word error with the word's span

### Requirement: Type checker reports all errors with source spans
All type errors SHALL include the span of the relevant AST node(s), enabling precise error messages.

#### Scenario: Type error includes location
- **WHEN** a type error is detected
- **THEN** the error includes at minimum the file name, line, and column of the offending node

#### Scenario: Effect error includes location and labels
- **WHEN** an effect propagation error is detected
- **THEN** the error identifies the missing effect label, the word that requires it, and the span of the word that lacks the declaration

### Requirement: Type checker processes definitions in dependency order
The type checker SHALL process word definitions such that a word's type is known before it is referenced in another definition's body. Forward references to mutually recursive words SHALL be handled by allowing pre-declaration.

#### Scenario: Forward reference resolved
- **WHEN** word `A` calls word `B` and `B` is defined after `A` in the source
- **THEN** the type checker successfully resolves `B`'s type when checking `A`

#### Scenario: Undeclared mutual recursion is an error
- **WHEN** two words are mutually recursive and neither has been pre-declared
- **THEN** the type checker reports a forward-reference error

### Requirement: Type checker annotates AST nodes with their inferred types
After type checking, every `AstNode` that represents a typed expression SHALL have its `type_info` field populated with the inferred `TypeInfo`.

#### Scenario: AST type info populated
- **WHEN** type checking completes successfully
- **THEN** every expression node in the AST has a non-None `type_info`

#### Scenario: Backends can read type info from AST
- **WHEN** a backend traverses the typed AST
- **THEN** it can access `node.type_info` to determine the type of any expression without re-running the type checker
