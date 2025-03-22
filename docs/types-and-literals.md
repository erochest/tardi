# Types and Literals in Tardi

Tardi supports several basic types and their corresponding literal representations. This document outlines the available types and how to use them in your Tardi programs.

## Supported Types

1. Integer
2. Float
3. Boolean
4. Character

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

## Type Inference

Tardi uses type inference to determine the type of a literal. You don't need to explicitly declare types; the language will infer them based on the literal representation.

## Stack Effect

When you use a literal in your Tardi program, it is pushed onto the stack. For example:

```
42 3.14 #t 'a'
```

This sequence will result in the following stack (from top to bottom):
```
'a'   (Character)
#t    (Boolean)
3.14  (Float)
42    (Integer)
```

You can then use these values in subsequent operations or manipulate them using Tardi's built-in functions and operators.
