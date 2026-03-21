## 1. AST Data Structures

- [x] 1.1 Define `Span` and `SourceId` types in `src/ast/span.rs`
- [x] 1.2 Define `AstNode` wrapper struct with `kind`, `span`, and `type_info: Option<TypeInfo>` fields
- [x] 1.3 Define `Node` enum variants: `Literal`, `Word`, `Quotation`, `Definition`, `Program`
- [x] 1.4 Define `TypeInfo` placeholder struct (to be filled by type checker)
- [x] 1.5 Implement recursive traversal / fold over `AstNode`
- [x] 1.6 Write unit tests for AST construction and traversal

## 2. Type Representation

- [x] 2.1 Define `StackType` with row variables and concrete types (`int`, `float`, `bool`, `str`, `char`)
- [x] 2.2 Define `TypeSig` struct: input row, output row, effect set
- [x] 2.3 Define `EffectSet` as a set of lowercase effect labels
- [x] 2.4 Implement effect set union operation
- [x] 2.5 Implement row variable unification
- [x] 2.6 Implement type signature parsing from string `"( S a -- S b | io )"`
- [x] 2.7 Write unit tests for type representation, unification, and effect union

## 3. Parser

- [x] 3.1 Create `src/parser/mod.rs` with `Parser` struct consuming `Vec<Value>`
- [x] 3.2 Implement parsing of literal tokens into `Literal` nodes (int, float, bool, str, char)
- [x] 3.3 Implement parsing of `Word` and `Symbol` tokens into `Word` nodes
- [x] 3.4 Implement parsing of `[ ... ]` delimiters into `Quotation` nodes
- [x] 3.5 Implement parsing of type signatures `( inputs -- outputs | effects )`
- [x] 3.6 Implement parsing of `: name ( sig ) body ;` definitions into `Definition` nodes
- [x] 3.7 Implement top-level parsing into `Program` node
- [x] 3.8 Implement error types with spans for all malformed token sequences
- [x] 3.9 Write unit tests for each node type including error cases
- [x] 3.10 Write integration tests parsing complete Tardi snippets

## 4. Type Checker

- [x] 4.1 Create `src/typechecker/mod.rs` with `TypeChecker` struct and type environment
- [x] 4.2 Implement word lookup: resolve `Word` nodes to their declared `TypeSig`
- [x] 4.3 Implement sequence type composition (compose two `TypeSig`s)
- [x] 4.4 Implement quotation type inference (infer type of `Quotation` node body)
- [x] 4.5 Implement definition checking (infer body type, unify with annotation)
- [x] 4.6 Implement effect propagation checking (caller declares superset of callee effects)
- [x] 4.7 Annotate each `AstNode` with its inferred `TypeInfo` after successful check
- [x] 4.8 Implement error reporting with spans for type mismatches and missing effects
- [x] 4.9 Handle forward references (pre-declaration mechanism for mutual recursion)
- [x] 4.10 Write unit tests for each checking rule
- [x] 4.11 Write integration tests checking complete word definitions

## 5. Pipeline Integration

- [ ] 5.1 Insert `Parser` stage in `src/compiler/mod.rs` after Pass 1 (macro expansion)
- [ ] 5.2 Insert `TypeChecker` stage after parsing, before code generation
- [ ] 5.3 Update Pass 2 (codegen) to consume the typed `AST` instead of raw `Vec<Value>`
- [ ] 5.4 Thread `Span` information through to VM error reporting
- [ ] 5.5 Verify REPL mode works with the new pipeline (warn on unannotated expressions)
- [ ] 5.6 Verify module loading works with the new pipeline

## 6. Annotate Built-in Rust Words

- [ ] 6.1 Add `type_sig: Option<TypeSig>` field to the builtin word registration mechanism
- [ ] 6.2 Annotate all stack ops: `dup`, `swap`, `rot`, `drop`, `clear`, `stack-size`
- [ ] 6.3 Annotate arithmetic ops: `+`, `-`, `*`, `/`
- [ ] 6.4 Annotate comparison ops: `==`, `<`, `>`, `not`
- [ ] 6.5 Annotate control flow ops: `apply`, `return`, `jump`, `break`, `continue`
- [ ] 6.6 Annotate return stack ops: `>r`, `r>`, `r@`
- [ ] 6.7 Annotate I/O words in `src/module/internal/io.rs` (with `| io` effect)
- [ ] 6.8 Annotate filesystem words in `src/module/internal/fs.rs` (with `| io` effect)
- [ ] 6.9 Annotate string words in `src/module/internal/strings.rs`
- [ ] 6.10 Annotate vector words in `src/module/internal/vectors.rs`
- [ ] 6.11 Annotate hashmap words in `src/module/internal/hashmaps.rs`
- [ ] 6.12 Annotate kernel and scanning words in `src/module/internal/kernel.rs`, `scanning.rs`

## 7. Annotate Bootstrap and Standard Library

- [ ] 7.1 Add type annotations to all words in `src/bootstrap/00-core-macros.tardi`
- [ ] 7.2 Add type annotations to all words in `src/bootstrap/01-stack-ops.tardi`
- [ ] 7.3 Add type annotations to all words in `src/bootstrap/02-core-ops.tardi`
- [ ] 7.4 Add type annotations to all words in `std/math.tardi`
- [ ] 7.5 Add type annotations to all words in `std/strings.tardi`
- [ ] 7.6 Add type annotations to all words in `std/vectors.tardi`
- [ ] 7.7 Add type annotations to all words in `std/hashmaps.tardi`

## 8. Tests and Validation

- [ ] 8.1 Update existing test fixtures in `tests/fixtures/` to include type annotations on definitions
- [ ] 8.2 Add test fixtures for type mismatch errors (wrong stack effect)
- [ ] 8.3 Add test fixtures for effect propagation errors (missing effect declaration)
- [ ] 8.4 Add test fixtures for undefined word errors
- [ ] 8.5 Verify all existing `cargo nextest run` tests pass with the new pipeline
- [ ] 8.6 Add test fixtures for quotation type inference
