## Context

`EffectSet` is a concrete `BTreeSet<EffectLabel>`. `TypeSig::effects` holds exactly one such set. This works for words whose effects are known at definition time, but breaks down for higher-order words that call a quotation argument: the quotation's effects belong to the caller but the callee cannot name them in its annotation.

The `quotation-type-definitions` change documents this gap in Decision 6 and Open Questions: `if`, `when`, and `unless` currently elide effect labels on their branch quotations, and `apply`/`dip`/`keep` cannot propagate quotation effects at all.

Effect variables solve this exactly as row variables solved the same problem for stack shapes: a lowercase identifier `e` in the `| ...` position ranges over unknown effect sets and is unified at call sites.

## Goals / Non-Goals

**Goals:**
- Add effect variable syntax to type signature strings: unknown lowercase identifiers in `| ...` are effect variables.
- Extend `EffectSet` to carry a set of variable names alongside concrete labels.
- Add effect variable unification in the type checker: bind `e → { io }` when a word typed `| e` is called with a quotation whose effects are `{ io }`.
- Propagate the union correctly: `| e f` binds two variables and requires the call site to declare their union.
- Re-annotate `if`, `when`, `unless`, `apply`, `dip`, `keep`, and the `bi`/`tri` family.

**Non-Goals:**
- Algebraic effect handlers (`handle` combinator). Effect variables are not the same as algebraic effects; they are a type-inference mechanism only.
- Inferring effect variables from word bodies; annotations remain explicit.
- Effect polymorphism beyond the `| e f ...` union syntax.

## Decisions

### Decision 1: Extend `EffectSet` with a variable set

**Choice:** Add `pub vars: BTreeSet<String>` to `EffectSet` alongside the existing `pub labels: BTreeSet<EffectLabel>`. An effect set with `vars = {"e"}` and `labels = {"io"}` represents `{ io } ∪ e`.

```rust
pub struct EffectSet {
    pub labels: BTreeSet<EffectLabel>,
    pub vars: BTreeSet<String>,   // NEW
}
```

**Alternative considered:** A new `EffectRow` enum with `Concrete` and `Variable` variants. Rejected because it would require replacing every `EffectSet` in the codebase with `EffectRow`. The field extension is additive and preserves all existing code paths (existing `EffectSet::empty()`, `union()`, etc. remain correct; `vars` just starts empty everywhere).

### Decision 2: Parser disambiguation — unknown identifiers become variables

**Choice:** In `parse_type_sig`, after splitting the `| ...` string on whitespace, each token is classified:
- Known built-in labels (`io`, `alloc`, `error`, `rand`, `time`) → concrete `EffectLabel`
- Unknown lowercase identifiers (one or more letters, no digits) → effect variable name added to `vars`
- Anything else → parse error

**Alternative considered:** A sigil like `'e` for variables. Rejected because the current design goal is to keep annotations readable and sigil-free, consistent with how row variables are just `S` (no sigil).

**Risk:** If a user invents a new concrete effect label that hasn't been registered, the parser will silently treat it as a variable. Mitigation: the type checker validates effect labels at definition time; an unknown label in a concrete position (where the checker expects a concrete set) would surface as an error. The distinction between "is this label known?" and "is this variable bound?" is handled at the unification layer.

### Decision 3: Unification strategy — substitution map per check

**Choice:** The type checker maintains a `HashMap<String, EffectSet>` (effect variable substitution) for the duration of each word check. When unifying two effect rows:
1. Expand variables using the substitution.
2. Concrete labels on both sides must match.
3. An unbound variable on one side is bound to the concrete part of the other side.
4. If both sides have variables, the variables are unified (same name required, or one is bound to the other's concrete set).

This is the same approach used for row variable unification, applied to effects.

**Alternative considered:** A global inference pass. Rejected as overly complex for annotations that are explicit; local per-call binding is sufficient.

### Decision 4: `| e f` means union of two variables

**Choice:** `| e f` in a return-position effect row represents `e ∪ f`. Both `e` and `f` are bound at the call site from the two branch quotations' effect sets. The call site's declared effects must be a superset of `e ∪ f` after substitution.

This enables the correct `if` type:
```
if : ( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )
```
When `if` is called with `[ when-true ]` (effects `{ io }`) and `[ when-false ]` (effects `∅`), `e` binds to `{ io }` and `f` binds to `∅`, so the call site must declare `{ io }`.

### Decision 5: `apply` uses two row variables and one effect variable

**Choice:**
```
apply : ( S [ S -- T | e ] -- T | e )
```
`T` is the second row variable introduced in the `quotation-type-definitions` design. `e` propagates the quotation's effects.

### Decision 6: Annotation updates use effect variables throughout

Higher-order words are re-annotated:
```
dip    : ( S a [ S -- T | e ] -- T a | e )
keep   : ( S a [ S a -- T | e ] -- T a | e )
2dip   : ( S a b [ S -- T | e ] -- T a b | e )
2keep  : ( S a b [ S a b -- T | e ] -- T a b | e )
bi     : ( S a [ S a -- T | e ] [ S a -- U | f ] -- T U | e f )
if     : ( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )
when   : ( S bool [ S -- S | e ] -- S | e )
unless : ( S bool [ S -- S | e ] -- S | e )
while  : ( S [ S -- S bool | e ] [ S -- S | f ] -- S | e f )
each   : ( S vec [ x -- | e ] -- S | e )
map    : ( S vec [ S x -- S y | e ] -- S vec | e )
```

The "effects elided" workaround documented in `quotation-type-definitions` Decision 6 is removed.

## Risks / Trade-offs

- **Variable vs label ambiguity at parse time**: An unregistered concrete effect label looks like a variable. → Mitigation: The checker catches misuse; the set of built-in labels is small and stable. A future `register-effect` mechanism can extend the known set.
- **Unification complexity grows**: Effect variable unification adds another dimension to the type checker's unification algorithm. → Mitigation: Keep effect unification strictly local to each call site; do not attempt global inference or constraint solving.
- **Annotation churn**: All higher-order word annotations change. → Mitigation: Annotations are silently consumed at runtime; no existing tests will break. The analysis pipeline will improve (fewer "effects not checked" gaps).

## Open Questions

- **Multiple variables in `bi`/`tri` family**: `bi` has type `( S a [S a -- T | e] [S a -- U | f] -- T U | e f )`. The `T` and `U` row variables and `e`, `f` effect variables interact. Unification must handle this correctly. The implementation should be tested with the full `bi`/`tri` family before considering the annotation update complete.
- **Effect variable scoping**: Should effect variables be scoped per-type-sig (fresh bindings each time a word is called) or per-analysis-run? Per-call-site fresh bindings are correct and simpler; this is the intended approach.
