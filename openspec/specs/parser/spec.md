## ADDED Requirements

### Requirement: Parser runs after macro expansion
The parser SHALL consume the post-macro-expansion token stream (a `Vec<Value>`) and produce an `AstNode` of kind `Program`. Macro expansion continues to operate on the raw token stream and is not visible to the parser.

#### Scenario: Simple token sequence becomes Program node
- **WHEN** the token stream `[Integer(1), Integer(2), Word("+")]` is parsed
- **THEN** the result is a `Program` node containing `[Literal(1), Literal(2), Word("+")]` in order

#### Scenario: Macro output is parsed normally
- **WHEN** macro expansion produces tokens and those tokens are passed to the parser
- **THEN** the parser treats them identically to hand-written tokens

### Requirement: Parser produces Literal nodes for value tokens
The parser SHALL convert all literal value tokens — integers, floats, booleans, strings, characters — into `Literal` AST nodes.

#### Scenario: Integer literal parsed
- **WHEN** the token stream contains `Integer(42)`
- **THEN** the parser produces `Literal(Integer(42))` with the correct span

#### Scenario: String literal parsed
- **WHEN** the token stream contains `String("hello")`
- **THEN** the parser produces `Literal(String("hello"))` with the correct span

#### Scenario: Boolean literal parsed
- **WHEN** the token stream contains `Boolean(true)`
- **THEN** the parser produces `Literal(Boolean(true))` with the correct span

### Requirement: Parser produces Word nodes for word and symbol tokens
The parser SHALL convert `Word` tokens and `Symbol { module, word }` tokens into `Word` AST nodes.

#### Scenario: Bare word parsed
- **WHEN** the token stream contains `Word("dup")`
- **THEN** the parser produces `Word { module: None, name: "dup" }`

#### Scenario: Module-qualified symbol parsed
- **WHEN** the token stream contains `Symbol { module: "vectors", word: "map" }`
- **THEN** the parser produces `Word { module: Some("vectors"), name: "map" }`

### Requirement: Parser produces Quotation nodes for bracket delimiters
The parser SHALL recognize `[` and `]` delimiters in the token stream and produce `Quotation` nodes containing the tokens between them.

#### Scenario: Empty quotation parsed
- **WHEN** the token stream contains `[ ]`
- **THEN** the parser produces `Quotation([])` with a span covering both brackets

#### Scenario: Nested quotations parsed
- **WHEN** the token stream contains `[ [ 1 ] ]`
- **THEN** the parser produces `Quotation([Quotation([Literal(1)])])` with correct nested spans

#### Scenario: Unmatched bracket is an error
- **WHEN** the token stream contains `[` with no matching `]`
- **THEN** the parser returns an error with a span pointing to the unmatched `[`

### Requirement: Parser produces Definition nodes for colon definitions
The parser SHALL recognize `: name ( type-sig ) ... ;` definitions and produce `Definition` AST nodes carrying the name, parsed type signature, and body.

#### Scenario: Simple definition parsed
- **WHEN** the token stream contains `: double ( S int -- S int ) dup + ;`
- **THEN** the parser produces `Definition { name: "double", type_sig: ..., body: [Word("dup"), Word("+")] }`

#### Scenario: Definition without type signature is an error
- **WHEN** the token stream contains `: double dup + ;` (missing type signature)
- **THEN** the parser returns an error indicating a type signature is required

#### Scenario: Unclosed definition is an error
- **WHEN** the token stream contains `: name ( S -- S ) word` with no `;`
- **THEN** the parser returns an error with a span pointing to the opening `:`

### Requirement: Parser preserves source spans on all nodes
Every AST node produced by the parser SHALL have a `span` accurately reflecting the source location of the token(s) it was built from.

#### Scenario: Span matches scanner position
- **WHEN** a token at line 3, column 5 is parsed into an AST node
- **THEN** that node's span has `start.line == 3` and `start.column == 5`

### Requirement: Parser errors include span and message
When the parser encounters a malformed token sequence, it SHALL return an error containing a source span and a human-readable message.

#### Scenario: Error includes location
- **WHEN** the parser encounters an unexpected token
- **THEN** the error includes the span of the unexpected token and a message describing what was expected
