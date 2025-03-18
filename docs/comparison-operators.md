# Comparison Operators

This document describes the comparison operators implemented in the Tardi language.

## Supported Operators

Tardi supports the following comparison operators:

- `==` (equal to)
- `!=` (not equal to)
- `<` (less than)
- `>` (greater than)
- `<=` (less than or equal to)
- `>=` (greater than or equal to)
- `!` (logical NOT)

## Implementation Details

### Scanner

The scanner recognizes these operators as distinct tokens. Each operator is treated as a separate `TokenType` in the scanner implementation.

### Compiler

The compiler translates these operators into appropriate VM instructions. Some operators (!=, <=, >=) are implemented as combinations of other basic operators:

- `!=` is implemented as `== !`
- `<=` is implemented as `> !`
- `>=` is implemented as `< !`

### VM

The VM implements these comparison operations:

- `equal`: Compares if two values are equal
- `not_equal`: Compares if two values are not equal (implemented as equal followed by NOT)
- `less`: Compares if a is less than b
- `greater`: Compares if a is greater than b
- `less_equal`: Compares if a is less than or equal to b (implemented as greater followed by NOT)
- `greater_equal`: Compares if a is greater than or equal to b (implemented as less followed by NOT)
- `not`: Performs logical NOT operation on the top stack item

## Usage

Comparison operators work with integers, floats, and booleans. When comparing different types, type coercion rules apply:

- Integers can be compared with floats (the integer is converted to a float)
- Booleans can only be compared with other booleans

Example usage:

```
1 2 <     // Pushes #t onto the stack (true)
2 2 ==    // Pushes #t onto the stack (true)
3 2 >=    // Pushes #t onto the stack (true)
1.5 1 >   // Pushes #t onto the stack (true)
#t !      // Pushes #f onto the stack (false)
```

## Error Handling

The VM includes error handling for type mismatches in comparison operations. If incompatible types are compared (e.g., trying to compare a boolean with a number), a `TypeMismatch` error is raised.

## Future Enhancements

Future versions may include:

- Support for comparing more complex data types (e.g., strings, lists)
- Short-circuiting boolean operations (AND, OR)
- Bitwise comparison operators
