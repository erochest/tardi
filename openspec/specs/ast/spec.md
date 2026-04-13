## ADDED Requirements

### Requirement: AST node types are defined
The system SHALL define an AST with node variants covering all syntactic forms of Tardi: literals, word references, quotations, function definitions, and top-level programs.

#### Scenario: Literal node exists
- **WHEN** the parser encounters a literal value (integer, float, boolean, string, character)
- **THEN** it produces an `AstNode` with kind `Literal` carrying the value

#### Scenario: Word node exists
- **WHEN** the parser encounters a bare word or a module-qualified symbol
- **THEN** it produces an `AstNode` with kind `Word` carrying an optional module name and the word name

#### Scenario: Quotation node exists
- **WHEN** the parser encounters a `[ ... ]` block
- **THEN** it produces an `AstNode` with kind `Quotation` containing an ordered list of child `AstNode`s

#### Scenario: Definition node exists
- **WHEN** the parser encounters a `: name ( type-sig ) ... ;` definition
- **THEN** it produces an `AstNode` with kind `Definition` carrying the name, type signature, and body as a list of child `AstNode`s

#### Scenario: Program node is the root
- **WHEN** a source file or REPL input is fully parsed
- **THEN** the result is an `AstNode` with kind `Program` containing all top-level nodes in order

### Requirement: Every AST node carries a source span
Every `AstNode` SHALL carry a `Span` recording the source location of the corresponding token(s), including source file identity, start position, and end position.

#### Scenario: Span preserved for literals
- **WHEN** a literal is parsed from source
- **THEN** its `AstNode.span` covers exactly the characters of that literal in the source

#### Scenario: Span preserved for words
- **WHEN** a word is parsed
- **THEN** its `AstNode.span` covers the word's lexeme in the source

#### Scenario: Span covers full quotation
- **WHEN** a quotation `[ ... ]` is parsed
- **THEN** its `AstNode.span` covers from the opening `[` to the closing `]`

#### Scenario: Span covers full definition
- **WHEN** a definition `: name ... ;` is parsed
- **THEN** its `AstNode.span` covers from `:` to `;`

### Requirement: AST nodes carry optional type information
Every `AstNode` SHALL have a field for optional `TypeInfo` that is `None` after parsing and may be filled by the type checker.

#### Scenario: Type info absent after parsing
- **WHEN** the parser produces an `AstNode`
- **THEN** `AstNode.type_info` is `None`

#### Scenario: Type info present after type checking
- **WHEN** the type checker successfully processes an `AstNode`
- **THEN** `AstNode.type_info` is `Some(TypeInfo)` containing the stack-effect type and effect set

### Requirement: AST supports traversal
The AST SHALL support recursive traversal, visiting every node in the tree.

#### Scenario: Visitor can walk all nodes
- **WHEN** a traversal is initiated from a `Program` node
- **THEN** every descendant node is visited exactly once in program order
