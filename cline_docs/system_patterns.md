# System Patterns

## System Architecture
The Tardi language system consists of the following main components:
1. Virtual Machine (VM)
   - Indirect Threaded Code (ITC) implementation
   - Function pointer table for operation dispatch
   - Stack-based execution model
2. Scanner (Tokenizer)
3. Compiler
4. Program Structure
5. REPL (Read-Eval-Print Loop) (planned)
6. File Executor (planned)
7. Module System (planned)

## Key Technical Decisions
1. Language Implementation: Rust
2. Version Control: jj with git backend
3. Build System: Cargo (Rust's package manager and build tool)
4. Test Framework: Cargo test with custom test harness
5. Task Runner: just (commands defined in Justfile)
6. VM Implementation: Indirect Threading (ITC)
   - Chosen for balance of performance and safety
   - Avoids unsafe code while maintaining reasonable performance
   - Uses function pointer table for operation dispatch
   - May be optimized to Direct Threading in future if needed

## Design Patterns
1. Stack-based architecture for the VM
2. Concatenative programming paradigm
3. Test-Driven Development (TDD) approach
4. SOLID principles in code organization
5. Indirect Threaded Code pattern for VM implementation
   - Operation indices stored in instruction stream
   - Function pointer table for operation lookup
   - Simple interpreter loop for execution

## Component Relationships
```mermaid
graph TD
    A[Source Code] --> B[Scanner]
    B --> |Iterator<Token>| C[Compiler]
    C --> D[Program]
    D --> |Constants Table| E[Virtual Machine]
    D --> |Instructions| E
    D --> |Op Table| E
    E --> F[Data Stack]
    E --> G[Return Stack]
    H[REPL] --> A
    I[File Executor] --> A
    J[Module System] --> A
    J --> C
```

## Project Structure
- `/src`: All source code
  - `main.rs`: Primary entrypoint for the executable
  - `lib.rs`: Primary entrypoint for the library, re-exports modules
  - `error.rs`: Defines different errors and bundles them into one enum
  - `vm/`: Virtual machine implementation
    - `mod.rs`: VM module definition and core functionality
  - `compiler/`
    - `mod.rs`: Compiler implementation
    - `program.rs`: Program structure
  - `scanner/`
    - `mod.rs`: Scanner implementation
    - `token.rs`: Token and TokenType definitions
- `/tests`: Integration tests
  - `/fixtures`: Test fixtures (*.tardi, *.stderr, *.stdout files)
- `/docs`: Documentation

## Library and Binary Structure
- The project is now structured as a library with a binary target
- `lib.rs` exposes the public interface of the library
- `main.rs` uses the library as a dependency
- Error handling is part of the library and used by the binary

## VM Architecture
- Indirect Threaded Code (ITC) implementation
  - Function pointer table stores operation implementations
  - Instruction stream contains indices into function table
  - Basic interpreter loop:
    1. Fetch next operation index
    2. Look up function pointer in table
    3. Execute operation
    4. Repeat
- Stack Management
  - Data stack for operation arguments and results
  - Return stack (planned) for control flow
- Error Handling
  - Custom error types for VM operations
  - Stack underflow/overflow protection
  - Type checking for operations
- Operations
  - Basic stack operations (push, pop)
  - 'lit' operation for loading constants
  - Arithmetic operations (planned)
  - Comparison operations (planned)
  - Stack manipulation primitives (planned)

## Error Handling
- Custom error types defined in `error.rs`
- Result type alias for error handling
- VM-specific error types for operation failures

## Testing Strategy
- Integration tests using custom test harness. Create an integration test before each major functionality is implemented.
- Unit tests written before implementation (TDD approach)
- Test fixtures for various scenarios
- VM operation tests:
  - Stack manipulation correctness
  - Error handling
  - Edge cases

## Code Organization
- SOLID principles
- Constants defined at the top of each file
- Minimal code duplication (refactor after three repetitions)
- Clear separation of VM components:
  - Core VM logic
  - Operation implementations
  - Stack management
