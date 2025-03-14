# Project Brief

## Overview

This project implements a small programming language called Tardi, short for tardigrade.

Currently the language is interpretted, and files have the extension `.tardi`.

The language is a stack-based concatenative language.

I'm writing this to get experience with implementing a language and to create a language I enjoy playing with and exploring ideas in.

## Road Map

1. Create a VM with core operations for retrieving literal values.

2. Add operations for stack manipulation.

3. Add a return stack, jumps, and operations for moving data between the data stack and the return stack.

4. Add operations for file and console IO.

5. Add operations for creating the core datatypes.

6. Add basic operations on integers, floating point numbers. rationals.

7. Add basic operations on strings.

8. Add basic operations on vectors.

9. Add basic operations on hashtables.

10. Add basic operations for FFI.

11. Add basic operations for green- and OS-threading.

12. Create a tokenizer/scanner that reads in words for the core VM operations and outputs an array of tokens.

13. Create a compiler that translates the array of tokens into bytecode for the VM to run.

14. Build in metaprogramming by allowing a word to trigger code that runs at scan/compile time and takes the array of tokens read so far, possibly reads future tokens, and then modifies the token array.

15. Initialize the system by reading an initializer file.

Longer term, I'd like to include these features:

- error messages with better error handling,
- conditionals,
- packages and modules,
- enums,
- structs,
- traits/protocols,
- number conversions,
- sets,
- regex,
- persistent data types,
- safe concurrency framework,
- Hindley-Milner type system,
- LLVM compiler frontend,
- and probably other things I'm not thinking about right now.

And for quality-of-life changes, I'd like to have these:

- documentation with tutorials,
- a website,
- a language server,
- and a tree-sitter parser.
