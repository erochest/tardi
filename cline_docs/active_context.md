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
- Implemented return stack operations (>r, r>, r@):
  - Added return stack to VM with overflow protection
  - Added scanner and compiler support for return stack operations
  - Created comprehensive test coverage and documentation
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
- Implemented comparison operators (==, !=, <, >, <=, >=, !):
  - Added scanner support for comparison operators
  - Added compiler implementation using basic operators and NOT
  - Implemented VM operations for comparisons
  - Added error handling for type mismatches
  - Added comprehensive tests for comparison operations
- Implemented shared value system using Rc<RefCell<Value>>:
  - Introduced SharedValue type alias and shared() helper function
  - Modified existing Value handling to use Rc<RefCell<Value>>
  - Updated stack operations to work with shared values
  - Adjusted existing operations (arithmetic, comparison) to handle shared values
  - Updated tests to verify shared value behavior
  - Monitored performance impact and addressed issues as they arose

## Next Steps
1. Add Character Values and Literals:
   - Implement Value::Char(char)
   - Add support for character literals in scanner and compiler
   - Implement character escape sequences
   - Add tests for character handling

3. Implement List Objects:
   - Add Value::List(Vec<Value>)
   - Implement basic list operations (<list>, append, prepend, concat, head)
   - Update tests for list functionality

4. Add String Objects and Literals:
   - Implement Value::String(String)
   - Add support for string literals in scanner and compiler
   - Implement basic string operations (<string>, >string, utf8>string, append, prepend, concat)
   - Add tests for string handling

5. Implement Function and Lambda Objects:
   - Create Function struct and Value::Function variant
   - Implement basic function-related words (<function>, <lambda>, curry)
   - Add support for function compilation
   - Update tests for function and lambda functionality

6. Add Comment Support:
   - Implement comment handling in scanner (// until end of line)

7. Create Initialization Script:
   - Implement script loading and execution
   - Create initial src/init.tardi script

8. Implement Compiler Words:
   - Add compile word for lambda/function compilation

9. Add Scanner/Parser Words:
    - Implement scan-word, scan-string, scan-tokens, scan-values

10. Implement Metaprogramming:
    - Add support for MACRO definitions
    - Implement macro expansion during compilation

11. Add Lambda and Function Literals:
    - Implement syntax for lambda and function literals
    - Update scanner and compiler to handle these literals

12. Regularly update documentation and memory bank files throughout the process

## Active Decisions and Considerations
- Using iterator pattern for scanner output
- Program structure contains constants, instructions, and op_table
- Separate error types for scanner, compiler, and VM phases
- Return stack implemented with a maximum capacity of 1024 items
- Return stack operations preserve the shared value system using Rc<RefCell<Value>>
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
- Monitoring performance impact of shared values (Rc<RefCell<Value>>)
- Postponing W register implementation until a specific use case arises
- Keeping initial string and list operations minimal, with plans for future enhancements
- Implementing features in the order presented in the function roadmap
- Maintaining existing testing strategy (input/output tests + unit tests)
- Planning for future enhancements such as traits/protocols and advanced iterators
