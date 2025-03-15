# Progress

## What Works
- Project initialization
- Basic project structure set up
- Documentation started (project brief, memory bank files)
- VM architecture decision made (Indirect Threading)

## What's Left to Build
1. VM with core operations for retrieving literal values
   - Function pointer table for ITC implementation
   - Basic interpreter loop
   - Initial literal value operations
2. Stack manipulation operations
3. Return stack, jumps, and operations for moving data between stacks
4. File and console IO operations
5. Core datatype creation operations
6. Basic operations on integers, floating point numbers, rationals
7. Basic string operations
8. Basic vector operations
9. Basic hashtable operations
10. FFI operations
11. Green- and OS-threading operations
12. Tokenizer/scanner
13. Compiler (token array to bytecode)
14. Metaprogramming capabilities
15. System initializer

## Current Status
- Early development stage
- Focus on implementing core VM operations
- Selected Indirect Threading (ITC) for VM implementation to balance performance and safety
- Planning initial implementation of literal value operations

## Known Issues
- No significant issues at this early stage

## Upcoming Milestones
1. Functional VM with basic operations
   - ITC-based interpreter loop
   - Function pointer table implementation
   - Basic literal value support
2. Working REPL for interactive testing
3. File execution capability
4. Basic standard library implementation

## Long-term Goals
- Error messages with better error handling
- Conditionals
- Packages and modules
- Enums
- Structs
- Traits/protocols
- Number conversions
- Sets
- Regex support
- Persistent data types
- Safe concurrency framework
- Hindley-Milner type system
- LLVM compiler frontend
- Performance optimization (possibly revisiting Direct Threading if needed)

## Quality-of-Life Improvements (Planned)
- Comprehensive documentation with tutorials
- Project website
- Language server
- Tree-sitter parser

## Next Steps
1. Implement core VM operations for literal values
   - Set up the function pointer table structure
   - Implement the basic ITC interpreter loop
   - Add initial literal value operations
2. Develop basic stack manipulation operations
3. Create simple test cases for VM functionality
4. Begin work on the REPL for interactive testing
