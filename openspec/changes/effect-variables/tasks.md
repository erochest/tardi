## 1. Effect Row Representation

- [ ] 1.1 Add `pub vars: BTreeSet<String>` to `EffectSet` in `src/types/mod.rs` alongside existing `labels`; update all `EffectSet` constructors (`empty`, `single`, `from_labels`) to initialize `vars` as an empty set
- [ ] 1.2 Update `EffectSet::union()` to union both `labels` and `vars` from both sides
- [ ] 1.3 Update `EffectSet::is_empty()` to return `true` only when both `labels` and `vars` are empty
- [ ] 1.4 Update `Display for EffectSet` to emit variable names (from `vars`, sorted) after concrete labels, space-separated

## 2. Parser and Display Updates

- [ ] 2.1 Update effect-token classification in `parse_type_sig`: after splitting `| ...` on whitespace, known built-in labels (`io`, `alloc`, `error`, `rand`, `time`) go into `labels`; unknown all-lowercase alphabetic identifiers go into `vars`; anything else is a `TypeSigParseError`
- [ ] 2.2 Verify `Display for TypeSig` already includes `| {effects}` correctly; confirm that variable names appear in output (they will if `Display for EffectSet` is updated in 1.4)
- [ ] 2.3 Verify round-trip: for a `TypeSig` with effect variables, `sig.to_string().parse::<TypeSig>() == Ok(sig)` holds

## 3. Type Checker — Effect Variable Unification

- [ ] 3.1 Add `type EffectSubst = HashMap<String, EffectSet>` (or equivalent) to `src/typechecker/mod.rs` for use during call-site checking
- [ ] 3.2 Implement `resolve_effect_set(es: &EffectSet, subst: &EffectSubst) -> EffectSet` — looks up each variable in the substitution and unions the resolved concrete sets with the existing labels; variables not in the substitution are carried forward as concrete empty contributions
- [ ] 3.3 Implement `unify_effect_rows(declared: &EffectSet, required: &EffectSet, subst: &mut EffectSubst) -> Result<(), TypeError>` — binds unbound variables in `required` to the concrete part of `declared`; checks that concrete labels in `required` are a subset of `declared`
- [ ] 3.4 Integrate effect variable binding into the call-site effect check: when a word with effect variables in its signature is called, create a fresh `EffectSubst`, bind variables from the quotation argument's resolved effects, then require the caller's declared effects to be a superset of the resolved outer effect row

## 4. Unit Tests — Representation and Parser

- [ ] 4.1 Test: `EffectSet::union()` unions both labels and vars from both sides
- [ ] 4.2 Test: `EffectSet::is_empty()` returns `false` when `vars` is non-empty even if `labels` is empty
- [ ] 4.3 Test: `( S [ S -- T | e ] -- T | e )` parses with `TypeSig.effects.vars = {"e"}` and no concrete labels
- [ ] 4.4 Test: `( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )` parses with outer `effects.vars = {"e", "f"}`
- [ ] 4.5 Test: `( S [ S -- T | e ] -- T | io e )` parses with `effects.labels = {"io"}` and `effects.vars = {"e"}`
- [ ] 4.6 Test: `io` in the `| io` position is still a concrete label (vars set is empty)
- [ ] 4.7 Test: display round-trip for `( S [ S -- T | e ] -- T | e )` and `( S [ S -- T | e ] -- T | io e )`

## 5. Unit Tests — Type Checker Unification

- [ ] 5.1 Test: calling a word typed `( S [ S -- T | e ] -- T | e )` with a `{ io }` quotation at a call site that declares `io` type-checks successfully
- [ ] 5.2 Test: calling the same word with a `{ io }` quotation at a call site that does not declare `io` produces a type error
- [ ] 5.3 Test: calling `if` typed `( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )` with a `{ io }` true branch and pure false branch requires the caller to declare `io`
- [ ] 5.4 Test: calling `if` with both branches pure produces no effect requirement for the caller
- [ ] 5.5 Test: two consecutive `apply` calls with `{ io }` and `{ alloc }` quotations require the caller to declare both effects

## 6. Re-annotate Higher-Order Stack Combinators (`src/bootstrap/01-stack-ops.tardi`)

- [ ] 6.1 Update `dip`:   `( S a [ S -- T | e ] -- T a | e )`
- [ ] 6.2 Update `2dip`:  `( S a b [ S -- T | e ] -- T a b | e )`
- [ ] 6.3 Update `3dip`:  `( S a b c [ S -- T | e ] -- T a b c | e )`
- [ ] 6.4 Update `keep`:  `( S a [ S a -- T | e ] -- T a | e )`
- [ ] 6.5 Update `2keep`: `( S a b [ S a b -- T | e ] -- T a b | e )`
- [ ] 6.6 Update `3keep`: `( S a b c [ S a b c -- T | e ] -- T a b c | e )`
- [ ] 6.7 Update `bi`:    `( S a [ S a -- T | e ] [ S a -- U | f ] -- T U | e f )`
- [ ] 6.8 Update `2bi`:   `( S a b [ S a b -- T | e ] [ S a b -- U | f ] -- T U | e f )`
- [ ] 6.9 Update `bi@`:   `( S a b [ S a -- T | e ] -- T T | e )`
- [ ] 6.10 Update `tri`:  `( S a [ S a -- T | e ] [ S a -- U | f ] [ S a -- V | g ] -- T U V | e f g )`
- [ ] 6.11 Update `tri1`, `tri2`, `tri3` with analogous effect variables

## 7. Re-annotate Higher-Order Control Flow (`src/bootstrap/02-core-ops.tardi`)

- [ ] 7.1 Update `if`:      `( S bool [ S -- T | e ] [ S -- T | f ] -- T | e f )`
- [ ] 7.2 Update `when`:    `( S bool [ S -- S | e ] -- S | e )`
- [ ] 7.3 Update `unless`:  `( S bool [ S -- S | e ] -- S | e )`
- [ ] 7.4 Update `while`:   `( S [ S -- S bool | e ] [ S -- S | f ] -- S | e f )`
- [ ] 7.5 Update `and`, `or`, `||` with effect variables if they accept quotation arguments
- [ ] 7.6 Update `deep-apply` and `while-continue?` with effect variables where applicable

## 8. Re-annotate Built-in Kernel Word (`src/module/internal/kernel.rs`)

- [ ] 8.1 Update `apply`: `( S [ S -- T | e ] -- T | e )`

## 9. Re-annotate Standard Library

- [ ] 9.1 Update `each`:      `( S vec [ x -- | e ] -- S | e )` in `std/vectors.tardi`
- [ ] 9.2 Update `each2`:     `( S vec vec [ x y -- | e ] -- S | e )` in `std/vectors.tardi`
- [ ] 9.3 Update `map`:       `( S vec [ S x -- S y | e ] -- S vec | e )` in `std/vectors.tardi`
- [ ] 9.4 Update `map2`:      `( S vec vec [ x y -- z | e ] -- S vec | e )` in `std/vectors.tardi`
- [ ] 9.5 Update `reduce`:    `( S vec a [ a x -- a | e ] -- S a | e )` in `std/vectors.tardi`
- [ ] 9.6 Update `filter`:    `( S vec [ x -- bool | e ] -- S vec | e )` in `std/vectors.tardi`
- [ ] 9.7 Update `all?`:      `( S vec [ x -- bool | e ] -- S bool | e )` in `std/vectors.tardi`
- [ ] 9.8 Update `any?`:      `( S vec [ x -- bool | e ] -- S bool | e )` in `std/vectors.tardi`
- [ ] 9.9 Update `partition`: `( S vec [ x -- bool | e ] -- S vec vec | e )` in `std/vectors.tardi`
- [ ] 9.10 Update internal higher-order helpers (`each-clear-apply`, `map-clear-apply`, `clean-apply`, `filter?`) with effect variables where meaningful in `std/vectors.tardi`
- [ ] 9.11 Update `each` and `map` in `std/hashmaps.tardi` with effect variables

## 10. Integration Tests

- [ ] 10.1 Add a `check_source` test: a word that calls `each` with an effectful `{ io }` quotation type-checks when the caller declares `io`, and fails when it does not
- [ ] 10.2 Add a `check_source` test: a word calling `if` with one effectful branch propagates that branch's effects to the caller
- [ ] 10.3 Run `cargo nextest run` to verify all existing tests still pass after all annotation updates
