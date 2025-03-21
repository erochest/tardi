# Types and Literals

## Basic Types

### Numbers
- **Integers**: Whole numbers
  ```
  42    // integer
  0     // zero
  -17   // negative integer
  ```
  
- **Floating Point**: Numbers with decimal points
  ```
  3.14  // float
  2.0   // float with zero decimal
  0.123 // float less than 1
  ```

  *Future number formats planned:*
  - Binary numbers (0b prefix)
  - Octal numbers (0o prefix)
  - Hexadecimal numbers (0x prefix)
  - Rational numbers (e.g., 3/4)
  - Exponential notation (e.g., 1e-10)
  - Floats with optional leading digit (e.g., .5)

### Booleans
Scheme-style boolean literals:
```
#t    // true
#f    // false
```

## Special Values
- **Error**: Represents an error condition
- **EOF**: End of file marker

## Notes
- Numbers currently support basic decimal formats
- More types and literals will be added as the language evolves
- All numeric literals can be preceded by a minus sign (-) for negative values
- Comments are denoted by double slashes (//)
- Semicolons (;) are reserved for function definitions (e.g., `: double 2 * ;`)

## Implementation Details
- All values in the VM are implemented as shared values using `Rc<RefCell<Value>>`
- This shared value approach allows for efficient memory management and enables future features like complex data structures and closures
- The shared value implementation is transparent to the end-user and doesn't affect the language syntax
