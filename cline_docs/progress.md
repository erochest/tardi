# Progress

## What Works
- Project initialization and basic structure
- Documentation:
  - Project brief, memory bank, types and literals, language syntax decisions
  - Stack manipulation operations (docs/stack-manipulation.md)
- VM architecture with Indirect Threading (ITC)
- Library structure (src/lib.rs, error handling, src/main.rs updates)
- Basic VM structure:
  - Function pointer table for ITC implementation
  - Basic interpreter loop
  - Stack operations (push/pop, iteration, display)
  - Error handling system with custom VMError types
- Scanner implementation:
  - Token and TokenType structures
  - Iterator interface
  - Position tracking
  - Basic literal value scanning (integers, floats, scheme-style booleans)
  - Error handling for invalid number formats and unexpected characters
  - Support for basic stack operation words (dup, swap, rot, drop)
- Compiler implementation:
  - Basic structure for compiling integers, floats, and booleans
  - Uses Program structure to generate bytecode
  - Support for compiling basic stack operations
- Program structure:
  - Manages constants, instructions, and op_table
  - Implements VMProgram trait
- VM enhancements:
  - Implementation of basic stack manipulation primitives (dup, swap, rot, drop)
- Test infrastructure:
  - Initial test suite using cargo-nextest
  - Integration tests for major features
  - System-specific tests (Scanner, Compiler, VM)
  - Line ending normalization
  - Comprehensive tests for basic stack operations
- TDD-focused workflow with emphasis on error case testing

## What's Left to Build
1. Enhance Scanner:
   - Support for arithmetic and comparison operators
   - Extended number formats (binary, octal, hex, rationals, exponential notation)
   - Error recovery for better reporting
2. Expand Compiler:
   - Support for arithmetic and comparison operators
   - Variable declaration and assignment
   - Function declarations and calls
   - Control flow structures (if/else, loops)
3. Enhance VM:
   - Arithmetic operations (add, sub, mul, div)
   - Comparison operations (eq, lt, gt, etc.)
   - Variable management and function call operations
4. Improve Program structure:
   - Methods for easier instruction and constant addition
   - Serialization/deserialization for programs
5. Implement module system
6. Develop basic standard library
7. File and console IO operations
8. Core datatype creation operations
9. Advanced data structures (vectors, hashtables)
10. FFI operations
11. Concurrency (Green- and OS-threading operations)
12. Metaprogramming capabilities
13. System initializer

## Current Status
- Basic scanner, compiler, and VM functionality implemented
- Program structure in place for managing bytecode
- Basic stack operations (dup, swap, rot, drop) implemented across all components
- Continuing work on expanding language capabilities
- Enhancing error handling and reporting across all components
- Improving test coverage and maintaining TDD-focused workflow

## Known Issues
- No significant issues at this stage

## Upcoming Milestones
1. Complete scanner enhancements
2. Implement basic arithmetic and comparison operations
3. Add support for variables and functions
4. Implement control flow structures
5. Working REPL for interactive testing
6. File execution capability
7. Basic module system
8. Initial standard library implementation

## Long-term Goals
- Comprehensive error handling and informative error messages
- Full support for all planned language features (enums, structs, traits/protocols)
- Advanced type system (possibly Hindley-Milner)
- Performance optimization (possibly revisiting Direct Threading)
- LLVM compiler frontend

## Quality-of-Life Improvements (Planned)
- Comprehensive documentation with tutorials
- Project website
- Language server
- Tree-sitter parser

## Next Steps
1. Enhance the Scanner:
   - Add support for arithmetic and comparison operators
   - Implement extended number formats
   - Improve error recovery and reporting
2. Expand the Compiler:
   - Implement support for arithmetic and comparison operators
   - Add variable declaration and assignment support
   - Add function declaration and call support
   - Implement control flow structures
3. Enhance the VM:
   - Add arithmetic and comparison operations
   - Add variable management and function call operations
4. Improve Program structure and serialization
5. Begin work on the module system
6. Continue improving test coverage and documentation:
   - Add documentation for arithmetic operations (when implemented)
   - Update stack-manipulation.md as new operations are added
