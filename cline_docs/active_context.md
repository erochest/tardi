# Active Context

## Current Work Focus
- Implementing compiler and scanner components
- Designing the Program structure for compiled code
- Implementing literal value handling through constants table
- Improving VM and test infrastructure

## Recent Changes
- Implemented basic VM structure with function pointer table and stack operations
- Created initial error handling system with VMError types
- Set up test infrastructure using cargo-nextest
- Implemented basic stack operations (push/pop)
- Decided on iterator-based token stream for scanner
- Chose 'Program' as the name for compiled code representation
- Planned separate error types for compilation phases
- Implemented scheme-style boolean literals (#t and #f)
- Added plans for extended number format support (binary, octal, hex, rationals, etc.)
- Created initial user-facing documentation for types and literals
- Decided on using double slashes (//) for comments in the language
- Implemented `Display` for `Value` in `src/vm/mod.rs`
- Added `stack_iter()` method to `VM` to iterate over stack values from bottom to top
- Updated `run_file` function in `src/main.rs` to use the new `stack_iter()` method
- Modified `validate_print_output` function in `tests/test_main.rs` to normalize line endings
- Improved test fixtures to handle different line ending formats

## Next Steps
1. Implement remaining scanner components
   - Add support for more token types (operators, identifiers, etc.)
   - Implement error recovery for better error reporting
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
5. Continue improving test coverage and infrastructure

## Active Decisions and Considerations
- Using iterator pattern for scanner output
- Program structure will contain constants, instructions, and op_table
- Separate error types for scanner, compiler, and VM phases
- lit operation will use constant table indices
- Decided on Indirect Threading (ITC) for VM implementation to avoid unsafe code while maintaining reasonable performance
- May revisit Direct Threading in the future if performance becomes a critical concern
- Scanner implementation focuses on literal values (integers, floats, booleans) as a starting point
- Error handling in scanner includes invalid number formats and unexpected characters
- Using scheme-style #t and #f for boolean literals
- Planning support for extended number formats:
  - Binary numbers (0b prefix)
  - Octal numbers (0o prefix)
  - Hexadecimal numbers (0x prefix)
  - Rational numbers (e.g., 3/4)
  - Exponential notation for floats (e.g., 1e-10)
  - Floats with optional leading digit (e.g., .5)
- Using double slashes (//) for comments in the language
- Reserving semicolons (;) for function definitions
- Stack iteration and display now consistently show values from bottom to top
- Test infrastructure now handles different line ending formats (CRLF vs LF)
