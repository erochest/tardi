# Arithmetic Operations

The language supports basic arithmetic operations for both integer and floating-point numbers. These operations follow standard mathematical rules and include automatic type coercion between integers and floats.

## Available Operations

| Operation | Symbol | Description | Example |
|-----------|--------|-------------|---------|
| Addition | `+` | Adds two numbers | `1 2 +` → `3` |
| Subtraction | `-` | Subtracts the top number from the second number | `3 2 -` → `1` |
| Multiplication | `*` | Multiplies two numbers | `2 3 *` → `6` |
| Division | `/` | Divides the second number by the top number | `6 2 /` → `3` |

## Type Coercion

When performing arithmetic operations between integers and floats:
- If either operand is a float, the result will be a float
- Integer operands are automatically converted to floats when needed
- All float values are displayed with a decimal point, even when they have no fractional part (e.g., `5.0`)

Examples:
```
2 1.5 +    // Integer + Float → 3.5 (Float)
3.0 2 *    // Float * Integer → 6.0 (Float)
5 2.5 /    // Integer / Float → 2.0 (Float)
```

## Error Handling

The following errors can occur during arithmetic operations:

1. Type Mismatch
   - Attempting to perform arithmetic with non-numeric values
   - Example: `1 #t +` will raise a type mismatch error

2. Division by Zero
   - Attempting to divide by zero (either integer or float)
   - Example: `1 0 /` will raise a division by zero error

3. Stack Underflow
   - Attempting to perform an operation with insufficient values on the stack
   - Example: `1 +` will raise a stack underflow error (needs two operands)

## Stack Effects

All arithmetic operations consume two values from the stack and push one result:

```
Before: ... a b
After:  ... (a op b)
```

Where:
- `a` and `b` are the operands
- `op` is one of: +, -, *, /
- The result is pushed back onto the stack

## Examples

```
// Integer arithmetic
1 2 +      // 3
3 4 -      // -1
2 3 *      // 6
10 2 /     // 5

// Float arithmetic
2.5 1.5 +  // 4.0
3.0 1.5 -  // 1.5
2.0 3.0 *  // 6.0
10.0 2.0 / // 5.0

// Mixed integer/float arithmetic
5 2.5 +    // 7.5
6 1.5 -    // 4.5
3 2.0 *    // 6.0
10 2.0 /   // 5.0
