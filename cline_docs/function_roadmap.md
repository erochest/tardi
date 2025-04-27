# Function Roadmap

This document outlines the implementation plan for various features of the Tardi language. Many of these features have already been implemented, and we'll focus on the remaining tasks and future enhancements.

## Implemented Features

### Shared Values
- Implemented `Rc<RefCell<Value>>` for shared values
- Defined `SharedValue` type and `shared()` function

### Return Stack
- Implemented separate return stack
- Added words: `>r` (to R), `r>` (from R), `r@` (copy R)

### Character Values and Literals
- Implemented `Value::Char(char)`
- Added support for character literals and Unicode characters

### List Objects
- Implemented `Value::List(Vec<Value>)`
- Added basic list operations: `<list>`, `append`, `prepend`, `concat`, `split-head!`

### String Objects and Literals
- Implemented `Value::String(String)`
- Added support for string literals, including triple-quoted strings
- Implemented basic string operations

### Function and Lambda Objects
- Implemented `Callable` enum with `BuiltIn(OpFn)` and `Fn(Function)`
- Added support for function and lambda creation
- Implemented compilation process for functions and lambdas

### Comments
- Implemented line comments starting with `//`

### Initialization Script
- Created `src/init.tardi` for system initialization
- Implemented script embedding and execution

### Compiler Words
- Implemented `compile` word for lambda/function compilation

### Scanner/Parser Words
- Implemented words: `scan-word`, `scan-string`, `scan-tokens`, `scan-values`

### Metaprogramming
- Implemented macro system with `MACRO:` definitions

### Lambda and Function Literals
- Added support for lambda literals with curly braces
- Implemented function literals with `:` syntax

## Remaining Tasks and Future Enhancements

1. File and Console I/O Operations
   - Implement basic file reading/writing capabilities
   - Add console input/output operations

2. Hashtable Implementation
   - Design and implement hashtable data structure
   - Add basic hashtable operations

3. FFI (Foreign Function Interface)
   - Design FFI system for Tardi
   - Implement basic FFI operations for calling external functions

4. Threading Support
   - Implement green threads for lightweight concurrency
   - Add support for OS-level threading

5. Enhanced Error Handling
   - Implement stack trace generation for error messages
   - Improve error context and readability
   - Implement error recovery mechanisms

6. Package and Module System
   - Design module system for code organization
   - Implement package management for Tardi projects

7. Advanced Type System
   - Consider implementing a more advanced type system (possibly Hindley-Milner)
   - Add support for user-defined types (enums, structs)

8. Traits/Protocols
   - Design and implement a trait/protocol system for polymorphism

9. Advanced Data Structures
   - Implement sets and persistent data types
   - Enhance existing data structures with more operations

10. Concurrency Framework
    - Develop a safe concurrency framework building on threading support

11. LLVM Integration
    - Research and plan LLVM integration for potential compilation support

12. Regular Expression Support
    - Implement regular expression operations and syntax

13. Performance Optimization
    - Profile VM performance and identify bottlenecks
    - Consider implementing Direct Threading if necessary
    - Optimize critical paths in the execution model

14. Language Server Protocol Implementation
    - Design and implement a language server for IDE integration

15. Documentation and Tutorials
    - Expand existing documentation to cover all language features
    - Create comprehensive tutorials for language usage

16. Development Tools
    - Implement a debugger for Tardi programs
    - Create additional development tools to aid in Tardi programming

## Implementation Considerations

- Maintain the existing TDD approach, creating tests before implementing new features
- Regularly update documentation as new features are added
- Monitor performance impacts of shared values and optimize where necessary
- Consider backward compatibility when implementing new features
- Regularly review and update this roadmap based on project progress and priorities
