# Project Brief

## Overview

This project implements a small programming language called Tardi, short for tardigrade.

Tardi is a stack-based concatenative language with support for macros, conditionals, recursive functions, and bootstrapping. The language is interpreted and files have the extension `.tardi`.

The implementation features a modular design with distinct components for scanning, compiling, and executing code, all orchestrated by a central `Tardi` struct. The system is bootstrapped through a series of Tardi scripts that define core macros and operations.

I'm writing this to get experience with implementing a language and to create a language I enjoy playing with and exploring ideas in.

## Completed Features

1. ✓ Core VM with operation dispatch and literal value handling
2. ✓ Stack manipulation operations (dup, swap, rot, drop, clear)
3. ✓ Return stack with data transfer operations (>r, r>, r@)
4. ✓ Arithmetic operations (+, -, *, /)
5. ✓ Comparison operations (==, <, >, !)
6. ✓ List operations (create-list, append, prepend, concat, split-head)
7. ✓ String operations (create-string, to-string, utf8-to-string, string-concat)
8. ✓ Function support (call, apply, return, exit, jump)
9. ✓ Macro system for compile-time code generation
10. ✓ Conditional execution
11. ✓ Recursive function support
12. ✓ Bootstrapping system for core language features
13. ✓ Scanner/tokenizer with support for words and literals
14. ✓ Compiler with bytecode generation
15. ✓ System initialization through bootstrap scripts

## Current Roadmap

Near-term goals:
1. File and console I/O operations
2. Hashtable operations
3. FFI (Foreign Function Interface)
4. Green- and OS-threading support
5. Enhanced error messages and error handling
6. Package and module system

Long-term features:
- Enums and structs
- Traits/protocols
- Number type conversions
- Sets and persistent data types
- Safe concurrency framework
- Hindley-Milner type system
- LLVM compiler frontend
- Regular expression support

Quality-of-life improvements:
- Comprehensive documentation with tutorials
- Project website
- Language server implementation
- Tree-sitter parser
- Development tools and IDE integration

## Architecture

The language implementation consists of several key components:

1. **Tardi**: Central orchestrator that manages the execution environment
2. **Scanner**: Tokenizes source code with support for macros
3. **Compiler**: Translates tokens to bytecode
4. **VM**: Executes bytecode using indirect threaded code
5. **Environment**: Manages the global state and function definitions
6. **Bootstrap System**: Initializes core language features through Tardi scripts

The system uses a modular design with clear separation of concerns, making it easy to extend and maintain. The bootstrap system allows core language features to be implemented in Tardi itself, promoting flexibility and self-hosting capabilities.
