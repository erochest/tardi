# Active Context

## Current Work Focus
- Enhancing scanner, compiler, and VM components
- Expanding language capabilities with new features
- Improving error handling and reporting
- Continuing TDD-focused workflow and expanding test coverage

## Development Workflow
1. Create integration test in `tests/fixtures` for new major features
2. Work iteratively on Scanner, Compiler, and VM systems:
   a. Create tests (including error cases)
   b. Implement functionality
   c. Run system-specific tests
3. Verify integration tests
4. Update documentation and memory bank
5. Review and adjust plans as needed

## Recent Changes
- Implemented basic compiler structure with support for integers, floats, and booleans
- Created Program structure for managing constants, instructions, and op_table
- Implemented VM with Indirect Threaded Code (ITC) and basic stack operations
- Improved scanner with support for integers, floats, and booleans
- Enhanced error handling across all components
- Improved test infrastructure and coverage
- Implemented basic stack operations (dup, swap, rot, drop) across all components:
  - Added scanner support for stack operation words
  - Added compiler support for stack operations
  - Implemented VM stack manipulation primitives
  - Added comprehensive tests for stack operations
- Implemented arithmetic operations (+, -, *, /):
  - Implemented std::ops traits for Value type
  - Added VM support for arithmetic operations
  - Added error handling for type mismatches and division by zero
  - Added comprehensive tests for arithmetic operations and error cases

## Next Steps
1. Implement Comparison Operations:
   - Scanner support for comparison operators
   - Compiler implementation for comparisons
   - VM operations for eq, lt, gt, etc.
   - Error handling for comparison operations
   - Integration tests for comparison functionality

4. Add Variable Support:
   - Scanner support for identifiers
   - Compiler implementation for variable declaration/assignment
   - VM operations for variable management
   - Error handling for variable operations
   - Integration tests for variable functionality

5. Implement Function Support:
   - Scanner support for function syntax
   - Compiler implementation for function declarations/calls
   - VM operations for function execution
   - Error handling for function operations
   - Integration tests for function functionality

6. Add Control Flow Structures:
   - Scanner support for if/else and loop syntax
   - Compiler implementation for control flow
   - VM operations for conditional execution and loops
   - Error handling for control flow
   - Integration tests for control flow functionality

7. Implement Extended Number Formats:
   - Scanner support for binary, octal, hex, rationals
   - Compiler implementation for extended numbers
   - VM operations for extended number types
   - Error handling for number formats
   - Integration tests for number format functionality

8. Add Module System:
   - Module syntax and loading
   - Compiler support for modules
   - VM support for module execution
   - Error handling for module operations
   - Integration tests for module functionality

9. Develop Basic Standard Library:
   - Core utility functions
   - Common operations library
   - Standard data structure implementations
   - Documentation and examples
   - Integration tests for standard library

10. Regularly update documentation and memory bank files

## Active Decisions and Considerations
- Using iterator pattern for scanner output
- Program structure contains constants, instructions, and op_table
- Separate error types for scanner, compiler, and VM phases
- Indirect Threading (ITC) for VM implementation to balance performance and safety
- May revisit Direct Threading in the future if performance becomes a critical concern
- Using scheme-style #t and #f for boolean literals
- Planning support for extended number formats (binary, octal, hex, rationals, exponential notation)
- Using double slashes (//) for comments in the language
- Reserving semicolons (;) for function definitions
- Stack iteration and display consistently show values from bottom to top
- Test infrastructure handles different line ending formats (CRLF vs LF)
- Need to implement a module system before developing the standard library
- Flexibility in adjusting the development plan as we progress
- Following Forth-style word handling with whitespace-delimited words that can start with any character
- Using helper functions to maintain consistent operation table management
