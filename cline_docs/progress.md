# Progress

## What Works
- Project initialization
- Basic project structure set up
- Documentation started (project brief, memory bank files)
- VM architecture decision made (Indirect Threading)
- Library structure implemented
  - Created src/lib.rs as the library entry point
  - Moved error handling to the library
  - Updated src/main.rs to use the library
- Basic VM structure implemented
  - Function pointer table for ITC implementation
  - Basic interpreter loop
  - Stack operations (push/pop)
  - Error handling system with custom VMError types
  - Initial test suite using cargo-nextest

## What's Left to Build
1. Scanner implementation
   - Token structures
   - Position tracking
   - Iterator interface
2. Program structure
   - Constants table
   - Operation table
   - Instruction format
3. Compiler implementation
   - Token consumption
   - Constant pooling
   - Literal handling
4. VM updates for Program integration
5. Stack manipulation operations
6. Return stack, jumps, and operations for moving data between stacks
7. File and console IO operations
8. Core datatype creation operations
9. Basic operations on integers, floating point numbers, rationals
10. Basic string operations
11. Basic vector operations
12. Basic hashtable operations
13. FFI operations
14. Green- and OS-threading operations
15. Metaprogramming capabilities
16. System initializer

## Current Status
- Shifting focus to compiler and scanner implementation
- Planning Program structure for compiled code
- Designing literal handling through constants table
- Selected Indirect Threading (ITC) for VM implementation to balance performance and safety

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
1. Implement scanner components
   - Create Token and TokenType structures
   - Implement Scanner with iterator interface
   - Add position tracking
2. Implement Program structure
   - Constants table management
   - Operation table construction
   - Instruction generation
3. Implement compiler
   - Token stream consumption
   - Constant pooling
   - Literal handling
4. Update VM to work with Program structure
   - Add lit operation
   - Modify VM to use Program object
