# Types and Literals in Tardi

Tardi supports several types and their corresponding literal representations. This document outlines the available types and how to use them in your Tardi programs.

## Supported Types

1. Integer
2. Float
3. Boolean
4. Character
5. String
6. List
7. Function

## Literal Representations

### Integer Literals

Integers are represented as whole numbers without a decimal point.

Examples:

```
42
-17
0
```

### Float Literals

Floats are represented as numbers with a decimal point.

Examples:

```
3.14
-0.5
2.0
```

### Boolean Literals

Booleans use Scheme-style notation:

- `#t` for true
- `#f` for false

### Character Literals

Characters are enclosed in single quotes.

Examples:

```
'a'
'Z'
'5'
'!'
```

#### Escape Sequences

Tardi supports the following escape sequences for special characters:

- `'\n'`: Newline
- `'\r'`: Carriage return
- `'\t'`: Tab
- `'\''`: Single quote
- `'\\'`: Backslash

#### Unicode Characters

Unicode characters can be represented in two ways:

1. Directly as UTF-8 characters:

   ```
   '🦀'
   '世'
   '界'
   ```

2. Using Unicode escape sequences:
   - `'\uXX'` for ASCII characters (where XX is a two-digit hexadecimal number)
   - `'\u{XXXX}'` for Unicode code points (where XXXX is a 1-6 digit hexadecimal number)

   Examples:

   ```
   '\u41'    // 'A'
   '\u7A'    // 'z'
   '\u{1F600}'  // '😀'
   '\u{1F4A9}'  // '💩'
   ```

### String Literals

Strings are enclosed in double quotes. For multi-line strings, use triple double quotes.

Examples:

```
"Hello, world!"
"Line 1\nLine 2"
"""
This is a
multi-line string
"""
```

Strings support the same escape sequences and Unicode representations as characters.

### List Literals

Lists are represented using curly braces `{}`.

Examples:

```
{ 1 2 3 }
{ "a" "b" "c" }
{ 1 "mixed" #t 3.14 }
```

### Function Literals

Function literals are defined using the `:` syntax:

```
: square ( n -- n^2 )
    dup *
;
```

Lambda expressions use square brackets:

```
[ dup * ]  // Anonymous function that squares a number
```

## Shared Value System

All values in Tardi are managed through a shared value system using `Rc<RefCell<Value>>`. This enables:

- Efficient sharing of values between different parts of the program
- Mutable access when needed
- Proper memory management through reference counting

## Type Inference

TODO: but no type checking. yet

Tardi uses type inference to determine the type of a literal. You don't need to explicitly declare types; the language will infer them based on the literal representation.

## Stack Effect

When you use a literal in your Tardi program, it is pushed onto the stack. For example:

```
42 3.14 #t 'a' "hello" { 1 2 3 } [ dup * ]
```

This sequence will result in the following stack (from top to bottom):

```
[ dup * ]     (Function)
{ 1 2 3 }     (List)
"hello"       (String)
'a'           (Character)
#t            (Boolean)
3.14          (Float)
42            (Integer)
```

You can then use these values in subsequent operations or manipulate them using Tardi's built-in functions and operators.

## Examples

```
// Using different types of literals
42 "The answer is: " concat >string concat  // "The answer is: 42"

// List manipulation
{ 1 2 3 } 4 push!  // { 1 2 3 4 }

// Function definition and application
: increment ( n -- n+1 )
    1 +
;
5 increment  // 6

// Lambda expression
[ 2 * ] 3 swap call  // 6
```

TODO: make sure all places that use `{...}` for lambdas are changed

Remember that all these literals and values are managed through the shared value system, allowing for efficient and safe manipulation of data in your Tardi programs.
