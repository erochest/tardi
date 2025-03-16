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

## Next Steps
1. Enhance the Scanner:
   - Add support for operators (arithmetic, comparison, etc.)
   - Implement identifiers for variables and functions
   - Complete the extended number formats (binary, octal, hex, rationals, exponential notation)
   - Implement error recovery for better error reporting

2. Expand the Compiler:
   - Add support for compiling operators
   - Implement variable declaration and assignment
   - Add support for function declarations and calls
   - Implement control flow structures (if/else, loops)

3. Enhance the VM with more operations:
   - Implement arithmetic operations (add, sub, mul, div)
   - Add comparison operations (eq, lt, gt, etc.)
   - Implement stack manipulation primitives (dup, swap, rot, drop)
   - Add operations for variable management and function calls

4. Improve the Program structure:
   - Add methods for easier instruction and constant addition
   - Implement serialization/deserialization for programs

5. Implement a module system

6. Develop a basic standard library

7. Enhance error handling and reporting across all components

8. Expand test coverage:
   - Add more comprehensive tests for the compiler
   - Create integration tests that cover the entire pipeline from scanning to execution

9. Regularly update documentation and memory bank files

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
