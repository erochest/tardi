## Why

Type signatures for higher-order words like `map`, `each`, `dip`, and `keep` currently use generic type variables (`a`, `b`) for quotation arguments, which provides no information about what those quotations consume or produce. Supporting quotation types like `[ x -- y ]` inline in signatures makes the type system expressive enough to document and eventually check higher-order contracts.

## What Changes

- The type signature parser gains support for `[ ... ]` notation inside signatures, producing `StackType::Quotation` values (which already exist in the data model but are unreachable from parsed strings).
- All higher-order built-in words and Tardi combinators have their annotations updated to use quotation types.
- The `TypeSig` display round-trips correctly for quotation-typed signatures.
- The type checker's `unify_stack_types` already handles `Quotation`-vs-`Quotation` unification; no logic changes are required there.

## Capabilities

### New Capabilities

- `quotation-type-parsing`: Parse `[ inputs -- outputs ]` tokens inside a type signature string into `StackType::Quotation(TypeSig)` values, with full support for nesting and effects.

### Modified Capabilities

- `stack-effect-types`: The type signature syntax is extended to allow quotation types as stack items; the existing grammar description needs to reflect the `[ ... ]` form.

## Impact

- `src/types/mod.rs`: `parse_type_sig` and `Row::from_tokens` rewritten to use a level-aware tokenizer.
- `src/module/internal/kernel.rs`: `apply` annotation updated.
- `src/bootstrap/01-stack-ops.tardi`: `dip`, `2dip`, `3dip`, `keep`, `2keep`, `3keep`, `bi`, `bi@`, `tri` and related combinators updated.
- `src/bootstrap/02-core-ops.tardi`: `if`, `when`, `unless`, `while`, `and`, `or`, `||` updated.
- `std/vectors.tardi`: `each`, `each2`, `map`, `map2`, `reduce`, `filter`, `all?`, `any?`, `partition`, and helpers updated.
- `std/hashmaps.tardi`: `each`, `map` updated.
- `std/strings.tardi`: `chunk-on`, `repeat-string` updated.
- Tests in `src/types/mod.rs` and `src/typechecker/mod.rs` extended to cover quotation-typed signatures.
