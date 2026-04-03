## Why

Higher-order words like `if`, `when`, `apply`, and `dip` call quotations whose effects belong to the caller — but the current type system has no way to express this. `EffectSet` is always a concrete set of labels; there is no effect variable that can range over an unknown set, so these words must either lie about their effects (elide them) or be untypeable. Effect variables close this gap the same way row variables closed the gap for stack polymorphism.

## What Changes

- Effect variable syntax is added to the `| ...` position of type signatures: a single lowercase identifier that does not match a known label name is treated as an effect variable (e.g. `| e`, `| e f` for the union of two variables).
- `EffectSet` is extended (or a parallel `EffectRow` type is introduced) to represent either a concrete set of labels or a variable (or a variable plus a concrete set).
- The type checker binds effect variables at call sites — when a word typed `( S [ S -- T | e ] -- T | e )` is called with a quotation whose effects are `{ io }`, `e` is bound to `{ io }` and the outer call site is required to declare `io`.
- Effect variable unification: two effect rows unify when their variables can be bound consistently and their concrete labels match.
- `if`, `when`, `unless`, `apply`, `dip`, `keep`, and the `bi`/`tri` family are re-annotated using effect variables. `if` becomes `( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )`, correctly expressing that the caller inherits the union of both branches' effects.
- The "effects elided" workaround in the current `if`/`when`/`unless` annotations is removed once this change lands.

## Capabilities

### New Capabilities

- `effect-variables`: Effect variables in type signatures — syntax, representation, binding, and unification rules for variables that range over effect sets.

### Modified Capabilities

- `tracked-effects`: The `| ...` position now allows effect variables in addition to concrete labels; propagation and composition rules are extended to cover variable rows.
- `stack-effect-types`: Type signature strings now accept effect variable identifiers in the `| ...` position; the parser and display round-trip for effect variables.

## Impact

- `src/types/mod.rs` — `EffectSet` / effect row type extended; `TypeSig` parser updated; `Display` updated.
- `src/typechecker/mod.rs` — unification extended to bind and propagate effect variables; call-site checking updated.
- `src/bootstrap/02-core-ops.tardi` — `if`, `when`, `unless`, `while` re-annotated.
- `src/bootstrap/01-stack-ops.tardi` — `dip`, `keep`, `bi`, `tri` and family re-annotated.
- `src/module/internal/kernel.rs` — `apply` re-annotated.
- `std/vectors.tardi`, `std/hashmaps.tardi` — `each`, `map`, `reduce`, `filter`, etc. re-annotated.
- No changes to the VM, compiler, or scanner.
