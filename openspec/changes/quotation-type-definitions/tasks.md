## 1. Level-Aware Type Signature Parser

- [ ] 1.1 Write `tokenize_sig_side(s: &str) -> Vec<String>` in `src/types/mod.rs` — a level-aware tokenizer that emits whitespace-separated tokens but treats `[ ... ]` spans (tracking `[`/`]` depth) as single tokens
- [ ] 1.2 Write `find_outer_arrow(s: &str) -> Option<usize>` — scans for `--` at bracket depth 0, returning the byte offset of the match (returns `None` if no outer `--` exists)
- [ ] 1.3 Rewrite `parse_type_sig` to use `find_outer_arrow` instead of `split_once("--")`
- [ ] 1.4 Rewrite `Row::from_tokens` to accept `&[String]` (owned tokens from the tokenizer) and dispatch on whether a token starts with `[` to call `parse_type_sig` recursively, producing `StackType::Quotation`
- [ ] 1.5 Wire `tokenize_sig_side` into `parse_type_sig` replacing the `split_whitespace` calls for input and output token collection

## 2. Tests for the Parser

- [ ] 2.1 Test: `( S vec [ x -- y ] -- S vec )` parses correctly — input row has `vec` then a `Quotation`, output row has `vec`
- [ ] 2.2 Test: `( S a [ S a -- S b ] -- S b a )` parses with a row-variable-bearing inner sig
- [ ] 2.3 Test: `( S vec [ x -- y | io ] -- S vec | io )` parses inner and outer effects correctly
- [ ] 2.4 Test: nested quotation type `( S [ [ x -- y ] -- z ] -- S z )` parses without error
- [ ] 2.5 Test: outer `--` is not confused with inner `--` in multi-nested sigs
- [ ] 2.6 Test: `( -- )` and `( -- | io )` still parse correctly (regression)
- [ ] 2.7 Test: `( a -- a a )` still parses correctly (regression — no row variable)
- [ ] 2.8 Test: display round-trip — `sig.to_string().parse::<TypeSig>() == Ok(sig)` for a quotation-typed sig
- [ ] 2.9 Test: malformed inner sig (e.g. unclosed `[`) returns `TypeSigParseError`

## 3. Annotate Built-in Words

- [ ] 3.1 Update `apply` in `src/module/internal/kernel.rs` to use a proper quotation type (e.g. `( S [ S -- S ] -- S | effect )`)
- [ ] 3.2 Update `scan-object-list` and `scan-value-list` in `src/module/internal/scanning.rs` if feasible

## 4. Annotate Bootstrap Combinators (`src/bootstrap/01-stack-ops.tardi`)

- [ ] 4.1 Update `dip`:   `( S a [ S -- S ] -- S a | effect )`
- [ ] 4.2 Update `2dip`:  `( S a b [ S -- S ] -- S a b | effect )`
- [ ] 4.3 Update `3dip`:  `( S a b c [ S -- S ] -- S a b c | effect )`
- [ ] 4.4 Update `keep`:  `( S a [ S a -- S b ] -- S b a | effect )`
- [ ] 4.5 Update `2keep`: `( S a b [ S a b -- S c ] -- S c a b | effect )`
- [ ] 4.6 Update `3keep`: `( S a b c [ S a b c -- S d ] -- S d a b c | effect )`
- [ ] 4.7 Update `bi`:    `( S a [ S a -- S b ] [ S a -- S c ] -- S b c | effect )`
- [ ] 4.8 Update `2bi`:   `( S a b [ S a b -- S c ] [ S a b -- S d ] -- S c d | effect )`
- [ ] 4.9 Update `bi@`:   `( S a b [ S a -- S c ] -- S c c | effect )`
- [ ] 4.10 Update `tri1`, `tri2`, `tri3` with quotation types
- [ ] 4.11 Update `tri` with quotation types

## 5. Annotate Bootstrap Control Flow (`src/bootstrap/02-core-ops.tardi`)

- [ ] 5.1 Update `if`:      `( S bool [ S -- S ] [ S -- S ] -- S | effect )`
- [ ] 5.2 Update `when`:    `( S bool [ S -- S ] -- S | effect )`
- [ ] 5.3 Update `unless`:  `( S bool [ S -- S ] -- S | effect )`
- [ ] 5.4 Update `while`:   `( S [ S -- S bool ] [ S -- S ] -- S | effect )`
- [ ] 5.5 Update `and`, `or`, `||` with quotation types
- [ ] 5.6 Update `deep-apply`, `while-continue?` with quotation types

## 6. Annotate Standard Library (`std/vectors.tardi`, `std/hashmaps.tardi`, `std/strings.tardi`)

- [ ] 6.1 Update `each`:    `( S vec [ x -- | effect ] -- S | effect )`
- [ ] 6.2 Update `each2`:   `( S vec vec [ x y -- | effect ] -- S | effect )`
- [ ] 6.3 Update `map`:     `( S vec [ x -- y | effect ] -- S vec | effect )`
- [ ] 6.4 Update `map2`:    `( S vec vec [ x y -- z | effect ] -- S vec | effect )`
- [ ] 6.5 Update `reduce`:  `( S vec a [ a x -- a | effect ] -- S a | effect )`
- [ ] 6.6 Update `filter`:  `( S vec [ x -- bool | effect ] -- S vec | effect )`
- [ ] 6.7 Update `all?`:    `( S vec [ x -- bool | effect ] -- S bool | effect )`
- [ ] 6.8 Update `any?`:    `( S vec [ x -- bool | effect ] -- S bool | effect )`
- [ ] 6.9 Update `partition`: `( S vec [ x -- bool | effect ] -- S vec vec | effect )`
- [ ] 6.10 Update `each` and `map` in `std/hashmaps.tardi`
- [ ] 6.11 Update internal higher-order helpers (e.g. `each-clear-apply`, `map-clear-apply`, `clean-apply`, `filter?`) with quotation types where meaningful

## 7. Tests for Quotation-Typed Annotations

- [ ] 7.1 Add a `check_source` integration test verifying that a definition calling `map` with a correctly-typed quotation type-checks successfully
- [ ] 7.2 Add a `check_source` integration test verifying that a definition calling `map` with a wrong-typed quotation (e.g. quotation producing two values when one is expected) produces a type error
- [ ] 7.3 Verify all existing `cargo nextest run` tests still pass after annotations are updated
