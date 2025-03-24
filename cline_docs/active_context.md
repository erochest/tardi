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
- Implemented Function and Lambda Objects:
  - Added Function struct and Value::Function variant with Callable enum
  - Implemented function declarations and calls
  - Added lambda expression support
  - Implemented jump operations for control flow
  - Added comprehensive tests for function and lambda operations
  - Added error handling for function-related operations
  - Created test fixtures for function and lambda functionality

- Introduced OpCode enum for VM operations:
  - Replaced string-based operation lookup with enum-based system
  - Added From<OpCode> for usize and TryFrom<usize> for OpCode implementations
  - Updated compiler to use OpCodes directly
  - Preserved op_map for future function support
  - Improved type safety and code clarity in VM implementation
  - All tests passing with new OpCode system

- Improved type conversion patterns:
  - Added From trait implementations for all Value variants (i64, f64, bool, char, String, Vec<SharedValue>)
  - Refactored compiler to use generic compile_constant<T: Into<Value>> method
  - Reduced code duplication in compiler implementation
  - Improved maintainability by leveraging Rust's type system

- Implemented string objects and literals:
  - Added Value::String variant
  - Added scanner support for string literals (both regular and triple-quoted)
  - Added compiler support for string literals and operations
  - Added string operations (<string>, >string, utf8>string, string-concat)
  - Added comprehensive tests for string functionality
  - Created documentation in docs/string-operations.md

- Implemented character values and literals:
  - Added Value::Char variant with support for Unicode characters
  - Added scanner support for character literals and escape sequences
  - Added compiler support for character literals
  - Added comprehensive tests for character handling
  - Created documentation for character types and literals
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
- Implemented List Objects:
  - Added Value::List(Vec<SharedValue>) variant
  - Implemented basic list operations (<list>, append, prepend, concat, split-head!)
  - Added scanner and compiler support for list operations
  - Updated VM to handle list operations
  - Added comprehensive tests for list functionality
  - Updated error handling for list operations
- Implemented Comment Support:
  - Added scanner support for line comments starting with //
  - Comments are skipped during tokenization
  - Added helper methods for end-of-line handling
  - Added comprehensive tests for comment handling
  - Updated scanner to handle comments in various contexts

## Next Steps
1. Add Comment Support:
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
- Function and Lambda implementation decisions:
  - Functions are stored as Value::Function with Callable enum
  - Lambdas are functions without names
  - Using return stack for function call management
  - Jump operations handle control flow
  - Function compilation maintains a stack of functions/lambdas
  - Functions can be defined and called at runtime
  - Error handling for type mismatches and invalid operations

- Using iterator pattern for scanner output
- Program structure contains constants, instructions, and op_table
- Separate error types for scanner, compiler, and VM phases
- Return stack implemented with a maximum capacity of 1024 items
- Return stack operations preserve the shared value system using Rc<RefCell<Value>>
- Character literals support both direct UTF-8 input and escape sequences
- Unicode support implemented via '\u{XXXX}' syntax for full Unicode range
- ASCII characters can use simplified '\uXX' syntax
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
