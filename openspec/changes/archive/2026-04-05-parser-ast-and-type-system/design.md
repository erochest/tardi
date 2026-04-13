## Context

Tardi currently compiles directly from a token stream (represented as `Value` objects) to a flat `Vec<usize>` bytecode with no intermediate representation. Source positions exist in the scanner but are not preserved past the token stage. There is no type information anywhere in the pipeline.

The proposed change introduces two new pipeline stages — a **Parser** and a **Type Checker** — between macro expansion and code generation. The typed AST they produce becomes the shared contract consumed by the existing VM backend and future native backends (LLVM, Cranelift).

```
BEFORE:
  Source → Scanner → Pass1 (macros) → Pass2 (codegen) → VM

AFTER:
  Source → Scanner → Pass1 (macros) → Parser → AST → TypeChecker → Backend(s)
                                                                     ├── VM (existing)
                                                                     └── Native (future)
```

## Goals / Non-Goals

**Goals:**
- Introduce an AST with source spans on every node
- Implement a parser that builds the AST from post-macro token streams
- Define a row-polymorphic stack-effect type system
- Implement tracked (propagating) effects as the first effect mechanism
- Type-check word definitions against their annotations; infer quotation types
- Annotate all existing builtins and bootstrap/stdlib words
- Lay groundwork for algebraic effect handlers (without implementing them)

**Non-Goals:**
- Native code backend (LLVM/Cranelift) — separate change
- Algebraic effect handlers — separate change, designed to extend this system cleanly
- LSP server — separate change, will consume the AST produced here
- Type inference for top-level word definitions (annotation required)
- Changing macro expansion — macros continue to operate on the token stream

## Decisions

### Decision 1: AST node types

Concatenative programs are fundamentally sequential; the AST is intentionally flat except at explicit nesting boundaries.

```rust
pub enum Node {
    Literal(Value),                          // 42, "hello", #t
    Word { module: Option<String>, name: String },
    Quotation(Vec<Node>),                    // [ ... ]
    Definition { name: String, body: Vec<Node>, type_sig: TypeSig },
    Program(Vec<Node>),
}

pub struct AstNode {
    pub kind: Node,
    pub span: Span,
    pub type_info: Option<TypeInfo>,         // filled by type checker
}

pub struct Span {
    pub source: SourceId,
    pub start: Pos,
    pub end: Pos,
}
```

**Alternatives considered:**
- Reuse `Value` as AST nodes — rejected because `Value` conflates runtime data with syntax; spans would be bolted on awkwardly and type info has nowhere to go.
- Separate `TypedAst` pass — rejected as premature; `Option<TypeInfo>` on each node allows the same type to serve both untyped (post-parse) and typed (post-typecheck) phases.

### Decision 2: Parser placement

The parser runs **after** macro expansion (Pass 1) and **before** code generation. Macros continue to operate on the raw `Value` token stream. Only the post-expansion token stream is parsed into AST nodes.

**Rationale:** Macro expansion can synthesize or reorder tokens arbitrarily. Parsing after expansion means the parser sees a stable, well-formed token sequence without needing to understand macro semantics. This matches how Lisp/Scheme macros work.

### Decision 3: Row-polymorphic stack-effect types

Type signatures use row polymorphism. The row variable `S` represents the rest of the stack beneath the arguments:

```
dup  : ( S a     -- S a a   )
swap : ( S a b   -- S b a   )
+    : ( S int int -- S int )
```

Composition is function composition. The type checker unifies row variables across word bodies.

**Alternatives considered:**
- Fixed-arity signatures without row variables — rejected because it makes higher-order words untyped; `dip`, `apply`, and all combinators require row polymorphism to type correctly.
- Full Hindley-Milner over the stack — considered for the future; for now, only quotations are inferred, top-level definitions are annotated.

### Decision 4: Tracked effects first, algebraic effects later

The effect system starts as **tracked effects**: labels that propagate upward via set union. A function calling something with `| io` must itself declare `| io`.

```
readline : ( file-like -- str | io )
process  : ( file-like -- str | io )   // io propagates from readline
```

The type representation is designed to support algebraic effect handlers later — when a `handle` combinator is added, it will consume an effect from the enclosed quotation's effect set. No runtime changes are required for tracked effects; algebraic handlers will require delimited continuations.

**Rationale:** Tracked effects deliver the core benefit (visible side effects in signatures) at low implementation cost. Designing the representation now to accommodate handlers avoids a breaking change later.

### Decision 5: Effect errors become effects, not Result types

Error conditions are modeled as effects (`| error`) rather than as `result<T>` return types. This is consistent with the overall effect philosophy and allows error handling to be unified with the handler mechanism when algebraic effects are added.

**Rationale:** This decision was reached after exploring Koka and Unison's approach. `result<T>` is a Rust/Haskell idiom that works against the concatenative stack model. Effect-based errors compose naturally with other effects.

### Decision 6: Annotation strategy for existing words

All word definitions — builtins in Rust, bootstrap Tardi scripts, and stdlib — require explicit type and effect annotations. Builtins gain a type signature field in their registration. Bootstrap and stdlib words gain inline annotation syntax.

Proposed annotation syntax for Tardi source:
```
: word-name ( S a -- S b | io )
    ... body ... ;
```

The type signature is parsed as part of the `:` definition macro.

## Risks / Trade-offs

- **Bootstrap annotation burden**: All three bootstrap files and the entire stdlib need annotations. This is a significant upfront cost but is the source of truth for the type system.
  → *Mitigation*: Do this incrementally; the type checker can warn on unannotated definitions initially rather than hard-erroring.

- **Row variable unification complexity**: Unifying row variables across composed words is non-trivial. Factor's type inferencer took significant effort.
  → *Mitigation*: Start with a simple unification algorithm; the annotation requirement on top-level words means the type checker only needs to infer inside quotation bodies, which are bounded.

- **Macro-generated code is untyped**: Code produced by macros is not directly visible to the type checker as typed AST — it's just a token stream that gets parsed. Macro output must typecheck the same as hand-written code.
  → *Mitigation*: This is the correct behavior. Macros are responsible for emitting well-typed token sequences.

- **`Value` used for both runtime and AST literals**: `Literal(Value)` in the AST reuses the runtime `Value` type. This creates a dependency between the AST layer and the value/runtime layer.
  → *Mitigation*: Acceptable for now; a dedicated `LiteralValue` enum can be introduced if the coupling becomes problematic.

## Open Questions

- **Effect handler syntax**: Not yet designed. The tracked effect system should be designed such that handler syntax can be added without changing existing annotations.
- **Effect label scoping**: Are effects global labels (like `io`, `alloc`) or can user code define new effect labels? Leaning toward user-definable effects for the algebraic handler phase.
- **Type annotation in REPL**: The REPL currently doesn't require function definitions. How should the type checker behave for unannotated REPL expressions?
- **Inference for top-level words**: Currently requiring annotations; should the type checker optionally infer and report the inferred type for unannotated definitions (useful during the annotation migration)?
