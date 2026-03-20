# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tardi is a stack-based concatenative programming language implemented in Rust. Files use the `.tardi` extension. The implementation features a modular design with distinct components for scanning, compiling, and executing code, orchestrated by a central `Tardi` struct. Core language features are bootstrapped through Tardi scripts in `src/bootstrap/`.

## Build and Development Commands

- Build: `just build` or `cargo build`
- Test: `just test [FILTERS]` or `cargo nextest run`
- Lint and format: `just lint` (runs clippy --fix, fmt, and commits)
- Check: `just check` (cargo check --tests)
- Watch tests: `just watch` (cargo watch with nextest)
- Install: `just install` (installs binary and copies std library)
- Update: `just update` (fetch updates and reinstall)
- Tasks: `just tasks` (find TODO/XXX comments in code)
- Update todos: `just update-todos` (update todo.txt from tasks)

## Running Tardi

- Execute file: `just run FILE [ARGS]` or `cargo run -- --print-stack FILE`
- REPL: `just repl [ARGS]` or `cargo run -- --print-stack`
- Initialize config: `cargo run -- config-init`

## Architecture

### Core Components

1. **Tardi** (`src/core/mod.rs`): Central orchestrator managing execution environment
2. **Scanner** (`src/scanner/`): Tokenizes source code with macro support
3. **Compiler** (`src/compiler/`): Translates tokens to bytecode
4. **VM** (`src/vm/`): Executes bytecode using indirect threaded code
5. **Environment** (`src/env.rs`): Manages global state and function definitions
6. **Value System** (`src/value/`): Type system with frozen/mutable variants and an I/O subsystem (`value/io/`) for reading and writing values
7. **Config** (`src/config.rs`): figment-based configuration management (TOML/YAML/JSON/env)

Memory model: values use `Rc<RefCell<T>>` (see `src/shared.rs`).

### Bootstrap System

The language self-hosts core features through scripts in `src/bootstrap/`:

- `00-core-macros.tardi`: Core macro definitions
- `01-stack-ops.tardi`: Stack manipulation operations
- `02-core-ops.tardi`: Core language operations

Standard library is in `std/` directory and gets installed to the user's data directory.

### Module System

Internal modules are in `src/module/internal/` with implementations for:

- File system operations (`fs.rs`)
- Hash maps (`hashmaps.rs`)
- I/O operations (`io.rs`)
- Kernel operations (`kernel.rs`, with loop semantics in `kernel/loop_word.rs`)
- String operations (`strings.rs`)
- Vector operations (`vectors.rs`)
- Runtime scanning/parsing (`scanning.rs`): `scan-value`, `scan-value-list`, `scan-object-list` — enables runtime DSL and metaprogramming
- Runtime reflection (`internals.rs`): `<function>`, `<predeclare-function>`
- Sandboxed execution (`sandbox.rs`): for definitions in the REPL. This should be renamed to avoid confusion with security sandboxing, which we also want.

### Standard Library

The `std/` directory contains Tardi-implemented library files:

- `vectors.tardi`: Comprehensive vector/list toolkit — constructors, iteration (`each`, `map`, `reduce`, `filter`), searching, slicing, analysis, etc.
- `hashmaps.tardi`: Hash map operations
- `strings.tardi`: String manipulation
- `math.tardi`: Basic math utilities (`1+`, `1-`, `abs`, `max`, `min`, `zero?`, `check-bounds`)

### Testing

Tests use `datatest-stable` for file-based testing. Test fixtures are in `tests/fixtures/` with corresponding `.tardi`, `.stderr`, `.stdout`, and `.status` files. The main test runner is in `tests/test_main.rs`. A `tests/modules/` directory holds module-level test cases (import cycles, etc.).

## Language Features

Tardi supports:

- Stack-based operations with a separate return stack
- Arithmetic and comparison operations
- Lists, strings, characters, hash maps, and vectors
- Functions, lambdas, and macros
- Conditional execution and loops (with break/continue)
- Module loading and imports (with cycle detection)
- Runtime scanning and metaprogramming
- Configurable REPL and file execution
