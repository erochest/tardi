## Why

Tardi currently compiles directly from tokens to bytecode with no intermediate representation, limiting error quality, preventing tooling, and making it impossible to target multiple backends. Adding a typed AST layer is the foundation for native code generation, a language server, and a static type system with algebraic effects.

## What Changes

- **NEW**: Parser stage transforms post-macro-expansion token stream into a structured AST
- **NEW**: AST carries source spans (line/column/offset) on every node, enabling precise error messages
- **NEW**: Stack-effect type system with row polymorphism — each word definition requires a type annotation `( S a -- S b )`
- **NEW**: Tracked effect system with declared effects on word definitions `( a -- b | io alloc )`
- **NEW**: Type checker validates annotations and infers types for quotations
- **NEW**: All built-in words and bootstrap definitions annotated with types and effects
- **MODIFIED**: Compilation pipeline gains a Parser → TypeChecker stage between macro expansion and code generation
- **MODIFIED**: Backends (VM and future native) consume typed AST instead of raw token stream

## Capabilities

### New Capabilities

- `ast`: The AST data structure — node types, span representation, traversal
- `parser`: Transforms token stream into AST after macro expansion
- `stack-effect-types`: Row-polymorphic type system for stack transformations
- `tracked-effects`: Declared effect labels that propagate through the call graph via set union
- `type-checker`: Validates word annotations, infers quotation types, checks effect propagation
- `typed-builtins`: Type and effect annotations for all built-in words and bootstrap definitions

### Modified Capabilities

<!-- No existing specs to modify — this is greenfield on top of existing infrastructure -->

## Impact

- **`src/compiler/`**: Parser stage inserted between Pass 1 (macro expansion) and Pass 2 (codegen)
- **`src/scanner/`**: Span/position data already tracked; needs to flow through to AST nodes
- **`src/bootstrap/`**: All three bootstrap files need type annotations on every word definition
- **`src/module/internal/`**: All built-in word registrations need type and effect signatures
- **`std/`**: Standard library definitions need type annotations
- **New crate or module**: `src/ast/` for AST types and `src/typechecker/` for type inference and checking
- **Future**: Typed AST becomes the input contract for LLVM/Cranelift backend (separate change)
- **Future**: Algebraic effect handlers extend tracked effects without changing the type representation (separate change)
