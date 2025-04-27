# Tech Context

## Technologies Used

### Core Technologies
- **Rust**: Primary implementation language
- **Cargo**: Package manager and build tool

### Development Tools
- **just**: Task runner for development workflows
- **jj**: Version control system with git backend
- **cargo-nextest**: Test runner with filtering capabilities
- **VSCode**: Recommended editor (based on current development setup)

## Development Setup

### Prerequisites
1. Rust toolchain (via rustup)
2. just command runner
3. jj version control system
4. cargo-nextest (for running filtered tests)

### Build System
- Cargo is used for building and managing dependencies
- Build configuration in `Cargo.toml`
- Custom build workflows defined in `Justfile`
- Bootstrap system for core language features

### Testing
- Run all tests: `just test`
- Run filtered tests: `just test FILTER`
- Test fixtures in `/tests/fixtures/`
  - `.tardi`: Source files
  - `.stderr`: Expected error output
  - `.stdout`: Expected standard output
- Test categories:
  - Arithmetic operations
  - Comparison operators
  - Stack operations
  - String operations
  - List operations
  - Function definitions
  - Conditionals
  - Bootstrapping
  - Character literals
  - Return stack operations

### Version Control Workflow
1. Start feature: `jj commit -m "DESCRIPTION"`
2. Work in progress: Regular commits
3. Complete feature: `jj squash`
4. Update description with story number
5. Push to GitHub:
   ```bash
   jj bookmark move --to @- main
   jj git push
   ```

## Technical Constraints

### Language Design
- Stack-based architecture with dual stack system:
  - Data stack for operation arguments and results
  - Return stack for control flow and function calls
- Concatenative programming paradigm
- File extension: `.tardi`
- Interpreted with bootstrapped core features
- Macro system for compile-time metaprogramming
- Support for recursive functions
- Conditional execution

### Core Components
1. **Tardi Orchestrator**
   - Central execution management
   - Environment coordination
   - Bootstrap process handling

2. **Virtual Machine**
   - Indirect Threaded Code (ITC) implementation
   - Function pointer table for operation dispatch
   - Stack-based execution model
   - Lambda function support
   - Macro execution capabilities

3. **Scanner/Compiler**
   - Token generation and management
   - Macro expansion support
   - Bytecode generation
   - Function compilation

4. **Environment**
   - Global state management
   - Function definitions
   - Macro storage and handling

### Error Handling
- Custom error types in `error.rs`
- Standardized Result type alias
- Stack safety checks
- Macro expansion error handling
- Function call error handling

### Performance Considerations
- VM optimization through Indirect Threaded Code
- Efficient stack operations
- Memory management via Rust's ownership system
- Shared state management using Rc/RefCell
- Bootstrap file loading optimization

## Dependencies
Currently minimal dependencies as shown in `Cargo.toml`:
- Standard Rust library
- Test framework dependencies
- Development tools (just, jj)
- Logging support

## Bootstrap System
- Core language features defined in Tardi
- Bootstrap files loaded in sorted order:
  1. `00-core-macros.tardi`: Core macro definitions
  2. `01-stack-ops.tardi`: Stack operation definitions
  3. `02-core-ops.tardi`: Core operation definitions
- Enables self-hosting capabilities
- Modular feature implementation

## Future Technical Considerations
1. LLVM integration for compilation
2. Language server protocol implementation
3. Tree-sitter grammar development
4. Documentation generation system
5. Package management system
6. File and console I/O operations
7. Hashtable implementation
8. FFI (Foreign Function Interface)
9. Green- and OS-threading support
10. Enhanced error messages with stack traces
