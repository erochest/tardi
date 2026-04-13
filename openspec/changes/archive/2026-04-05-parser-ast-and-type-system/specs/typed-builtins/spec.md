## ADDED Requirements

### Requirement: All built-in Rust words have type and effect signatures
Every word registered in the op-table from Rust code (builtins in `src/module/internal/`) SHALL have an associated `TypeSig` declaring its stack-effect type and effect set. These signatures are registered at startup and are authoritative for the type checker.

#### Scenario: Stack primitive typed
- **WHEN** `dup` is registered
- **THEN** its type signature `( S a -- S a a )` is associated with it in the type environment

#### Scenario: Arithmetic word typed
- **WHEN** `+` is registered
- **THEN** its type signature `( S int int -- S int )` is associated with it

#### Scenario: IO word typed with effect
- **WHEN** an I/O word such as `print` is registered
- **THEN** its type signature includes `| io` in the effect set

#### Scenario: Type checker uses builtin signatures
- **WHEN** user code calls a builtin word
- **THEN** the type checker uses the registered signature to type-check the call site

### Requirement: All bootstrap word definitions include type annotations
Every word defined in `src/bootstrap/00-core-macros.tardi`, `01-stack-ops.tardi`, and `02-core-ops.tardi` SHALL include an inline type annotation as part of its definition.

#### Scenario: Bootstrap stack combinator annotated
- **WHEN** `dip` is defined in bootstrap
- **THEN** it includes a type signature such as `( S a [ S -- S' ] -- S' a )`

#### Scenario: Bootstrap definition without annotation is an error
- **WHEN** a bootstrap word is defined without a type signature
- **THEN** the type checker reports a missing annotation error during bootstrap loading

### Requirement: All standard library words include type annotations
Every word defined in `std/` (vectors.tardi, hashmaps.tardi, strings.tardi, math.tardi) SHALL include an inline type annotation.

#### Scenario: Standard library word annotated
- **WHEN** `vectors::map` is defined in `std/vectors.tardi`
- **THEN** it includes a type signature covering its stack transformation and any effects

#### Scenario: Type checker uses stdlib annotations
- **WHEN** user code calls `vectors::map`
- **THEN** the type checker checks the call site against the annotated signature

### Requirement: Type annotation format is consistent across builtins and user code
The same type annotation syntax `( inputs -- outputs | effects )` SHALL be used for both inline Tardi definitions and the Rust-registered builtin signatures. Rust-registered signatures use the same string representation, parsed at registration time.

#### Scenario: Rust builtin signature parses correctly
- **WHEN** a builtin is registered with signature string `"( S a -- S a a )"`
- **THEN** the signature is parsed into the same internal `TypeSig` representation as an inline annotation

#### Scenario: Invalid builtin signature panics at startup
- **WHEN** a builtin is registered with a malformed signature string
- **THEN** the runtime panics at startup with a message identifying the malformed signature

### Requirement: Unannotated builtins are flagged during development
During the annotation migration period, words without type signatures SHALL produce a warning (not an error) when first called, identifying the word and its location of definition.

#### Scenario: Warning on unannotated word call
- **WHEN** the type checker encounters a call to a word with no registered type signature
- **THEN** it emits a warning including the word name and proceeds with an unknown type (allowing the program to still run)
