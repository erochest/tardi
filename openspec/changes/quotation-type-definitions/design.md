## Context

Tardi's type system has `StackType::Quotation(Box<TypeSig>)` fully defined — it is produced by the type checker's `infer_quotation` when it sees a `[ body ]` node, and `unify_stack_types` already handles quotation-vs-quotation unification. However, `parse_type_sig` and `Row::from_tokens` are simple whitespace-split implementations that cannot produce `StackType::Quotation` from a string. The result is that quotation types exist at runtime but can never appear in a parsed type annotation.

The current parsing pipeline:
1. Strip outer `( ... )` parens
2. `split_once("--")` to find the input/output boundary
3. `split_whitespace()` each side into token slices
4. `Row::from_tokens` maps each token to a `StackType` via `StackType::from_token`

Step 2 breaks immediately when a nested `[ x -- y ]` appears in the signature — `split_once` fires on the inner `--`. Steps 3–4 have no concept of multi-token spans.

## Goals / Non-Goals

**Goals:**
- Parse `[ inputs -- outputs | effects ]` spans inside type signatures as `StackType::Quotation`.
- Support full nesting: `[ S a [ b -- c ] -- S d ]` is valid.
- Round-trip: `sig.to_string().parse::<TypeSig>()` succeeds for any produced type.
- Update all higher-order word annotations to use quotation types.

**Non-Goals:**
- Call-site quotation checking (verifying that a runtime quotation value matches a declared quotation parameter type). This requires connecting the type checker's inference path to the call-site lookup path and is a separate concern.
- Type inference for quotation types in function bodies (beyond what `infer_quotation` already does).
- Changes to the VM or compiler.

## Decisions

### Decision 1: Level-aware tokenizer replaces whitespace split

**Choice:** Replace `split_whitespace().collect::<Vec<&str>>()` with a custom `tokenize_sig_side(s: &str) -> Vec<String>` that emits tokens and bracketed spans as single items, tracking `[`/`]` depth.

For example, `"S vec [ x -- y ] bool"` yields `["S", "vec", "[ x -- y ]", "bool"]`.

**Alternative considered:** Regex-based span extraction. Rejected because Tardi has no regex dependency and regex cannot handle arbitrary nesting.

### Decision 2: Level-aware `--` finder for the outer split

**Choice:** Before splitting on `--`, scan the inner string character-by-character tracking `[`/`]` depth. Only split on `--` found at depth 0.

**Alternative considered:** Parsing the whole signature with a recursive descent parser from scratch. This is more correct but much larger. The level-aware split is sufficient because `[` is the only nesting delimiter in type signatures.

### Decision 3: Recursive `StackType::from_token` for bracketed spans

**Choice:** In `Row::from_tokens`, when a token starts with `[`, strip the outer brackets and call `parse_type_sig` recursively to produce a `StackType::Quotation`.

The existing `StackType::from_token` becomes `StackType::from_str_token(s: &str) -> Self` that handles the single-token fast path. A new `StackType::from_sig_token(s: &str) -> Result<Self, ...>` handles the `[...]` case.

### Decision 4: Quotation notation uses `[ ... ]`, not `( ... )`

**Choice:** Quotation types are written as `[ inputs -- outputs ]` inside signatures, consistent with how quotations look in Tardi source code (`[ body ]`) and with the existing `StackType::Quotation` Display impl.

**Alternative considered:** `( ... )` parens for quotation types (as some other stack languages do). Rejected because `( ... )` is already the outer delimiter for type signatures, creating ambiguity.

### Decision 5: Higher-order annotations keep explicit `S` row variable

Higher-order words need `S` to express what stack context passes through to the quotation. Simple words continue using the optional short form. So `map` becomes:

```
( S vec [ S x -- S y ] -- S vec )
```

where `S` is the rest of the stack (including any extra context the quotation reaches into), `vec` is the input vector, and `[ S x -- S y ]` is the element transform. The `S` inside the quotation type is intentional: it allows quotations that reach past the top element, such as a running-total accumulator sitting below the current element on the stack.

### Decision 6: `apply` uses two row variables; `if`/`when`/`unless` effects elided

`apply`'s correct type is `( S [ S -- T | e ] -- T | e )`, using a second row variable `T` to express that the stack shape may change after the quotation runs, and an effect variable `e` to propagate the quotation's effects to the caller. This is also the correct shape for `dip`, `keep`, and `if`.

However, `if` reveals a limitation of the current `EffectSet` model. Its ideal type is:

```
( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )
```

where `e` and `f` are *effect variables* (ranging over effect sets, not individual labels) and `e f` denotes their union. This is necessary because the two branches often have different effect sets — notably, `when` passes `[ ]` (effects `∅`) as the else branch, so requiring `e = f` would make `when` untypeable for any effectful true branch. The union `e ∪ f = e ∪ ∅ = e` correctly allows the effects of the true branch to propagate.

**Effect variables do not exist in the current type system.** `EffectSet` is always a concrete set of labels; there is no mechanism to write `e` or `f` as a variable in a signature string and have the checker bind and unify them.

For this change, annotations for `if`, `when`, and `unless` will **elide the effect labels on the branch quotations** and document this limitation explicitly. For example:

```
if   : ( S bool [ S -- T ] [ S -- T ] -- T )
when : ( S bool [ S -- S ] -- S )
```

The effect propagation for these words is still handled at the call site (the caller's declared effects must be a superset of what `if`/`when`/`unless` actually executes). See the Open Questions section for the effect variable follow-up.

## Risks / Trade-offs

- **Annotation effort is substantial**: ~20 higher-order words across three files need updated signatures. Annotations are mechanical but numerous.
  → Mitigation: The type checker doesn't enforce annotations during compilation (parallel pipeline), so annotation errors produce type errors in analysis mode only — existing tests will continue passing even with partially-updated annotations.

- **Recursive parsing complexity**: `parse_type_sig` becomes mutually recursive with `Row::from_tokens`. Stack overflow is theoretically possible for deeply nested types.
  → Mitigation: Type annotations are written by humans and will never be deeply nested in practice; no explicit depth limit is needed.

- **`split_once("|")` for effects also needs depth awareness**: If an effect label happened to contain `[...]`, the effects split could misbehave. In practice effects are simple lowercase identifiers, so this is not a real risk. The `|` split can remain a simple `split_once`.

## Open Questions

- **`apply` row variable choice**: `apply` will be annotated as `( S [ S -- T | e ] -- T | e )`. The consequences for `dip`, `keep`, and the `bi`/`tri` family are documented in Decision 6. The annotation update tasks should be revised to use two-row-variable forms throughout.

- **Effect variables** *(follow-on change)*: The `if`/`when`/`unless` annotation limitation (Decision 6) points to a missing feature: effect variables `e`, `f` that range over effect sets, analogous to row variables `S`, `T` that range over stack shapes. A future change should:
  - Add effect variable syntax to `TypeSig` (single lowercase identifiers in the `| ...` position that don't match known label names)
  - Extend `EffectSet` or introduce a parallel `EffectRow` type for the variable case
  - Add effect variable unification to the type checker, binding `e` to the concrete effect set at each call site and computing union for `e f`
  - Re-annotate `if` as `( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )` and `when`/`unless` accordingly

  Until then, effect labels on branch quotations in `if`, `when`, and `unless` are elided in annotations.
