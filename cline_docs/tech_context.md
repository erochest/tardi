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

### Testing
- Run all tests: `just`
- Run filtered tests: `cargo nextest run FILTER`
- Test fixtures in `/tests/fixtures/`
  - `.tardi`: Source files
  - `.stderr`: Expected error output
  - `.stdout`: Expected standard output

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
- Stack-based architecture
- Concatenative programming paradigm
- File extension: `.tardi`
- Initially interpreted (compilation planned for future)

### Error Handling
- Custom error types in `error.rs`
- Standardized Result type alias
- Error messages with stack traces (planned)

### Performance Considerations
- VM optimization requirements
- Stack operation efficiency
- Memory management strategy

## Dependencies
Currently minimal dependencies as shown in `Cargo.toml`:
- Standard Rust library
- Test framework dependencies
- Development tools (just, jj)

## Future Technical Considerations
1. LLVM integration for compilation
2. Language server protocol implementation
3. Tree-sitter grammar development
4. Documentation generation system
5. Package management system
