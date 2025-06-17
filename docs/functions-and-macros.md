# Functions and Macros in Tardi

Tardi supports both functions (runtime execution) and macros (compile-time code generation). This document covers the modern syntax and capabilities for defining and using both.

## Function Definition

### `:` - Function Definition Syntax

Functions are defined using the `:` syntax, which is implemented as a macro:

```tardi
: function-name ( stack-effect-comment )
    function-body
;
```

The `:` macro automatically handles:
- Function pre-declaration (for forward references)
- Compilation of the function body
- Registration in the environment

#### Examples

```tardi
// Simple function
: square ( n -- n^2 )
    dup *
;

// Function with multiple operations
: average ( a b -- avg )
    + 2 /
;

// Function using conditionals
: abs ( n -- |n| )
    dup 0 < [ -1 * ] when
;

// Recursive function
: factorial ( n -- n! )
    dup 1 <= [ drop 1 ] [
        dup 1 - factorial *
    ] if
;
```

## Lambda Expressions

### `[ ... ]` - Lambda Syntax

Lambdas are anonymous functions created using square brackets:

```tardi
[ dup * ]  // Lambda that squares a number
```

Lambdas are commonly used with:
- Conditional operations (`if`, `when`)
- Loop constructs (`while`)
- Higher-order functions (`map`, `each`, `keep`)

#### Examples

```tardi
// Using lambda with conditionals
5 0 > [ "positive" ] [ "not positive" ] if

// Using lambda with stack combinators  
42 [ 2 * ] keep  // Applies lambda but keeps original value

// Using lambda with iteration
{ 1 2 3 4 5 } [ dup * ] map  // Square each element
```

## Function Composition and Combinators

### `apply ( ... lambda -- ... )`
Executes a lambda or function with the current stack.

```tardi
5 [ dup * ] apply  // Results in: 25
```

### Stack Preservation Combinators

#### `keep ( ... x lambda -- ... x' x )`
Applies lambda to value but preserves the original.

```tardi
10 [ 3 + ] keep  // Results in: 13 10
```

#### `dip ( ... x lambda -- ... lambda-result x )`
Applies lambda to stack below the top element.

```tardi
1 2 3 [ + ] dip  // Results in: 3 3 (adds 1+2, keeps 3)
```

#### `2dip ( ... x y lambda -- ... lambda-result x y )`
Applies lambda to stack below the top two elements.

```tardi
1 2 3 4 [ + ] 2dip  // Results in: 3 3 4 (adds 1+2, keeps 3 4)
```

## Macro Definition

### `MACRO:` - Macro Definition Syntax

Macros execute at compile time and generate code:

```tardi
MACRO: macro-name
    compile-time-code
;
```

Macros have access to:
- The compilation environment
- Scanning functions to read input tokens
- Code generation capabilities

#### Built-in Scanning Functions

- `scan-value` - Scan a single token/value
- `scan-object-list` - Scan tokens until delimiter and compile them
- `push!` - Add generated code to output

#### Examples

```tardi
// Simple macro that escapes the next token
MACRO: \
    dup scan-value swap push!
;

// Lambda definition macro
MACRO: [
    dup
    ] scan-object-list compile
    swap push!
;

// Vector literal macro
MACRO: {
    dup
    } scan-object-list
    swap push!
;

// Function definition macro
MACRO: :
    scan-value
    dup <predeclare-function>
    \ ; scan-object-list compile
    <function>
;
```

## Advanced Function Patterns

### Higher-Order Functions

Functions that operate on other functions:

```tardi
: twice ( lambda -- )
    dup apply apply
;

[ 2 * ] twice  // Applies the doubling function twice
```

### Partial Application

```tardi
: add-n ( n -- lambda )
    [ + ] 
;

5 add-n  // Creates a function that adds 5
```

### Function Composition

```tardi
: compose ( f g -- composed )
    [ swap apply apply ]
;

[ 2 * ] [ 1 + ] compose  // Creates function that adds 1 then doubles
```

## Control Flow Functions

### `if ( condition true-lambda false-lambda -- )`
Standard conditional execution.

```tardi
n 0 > [ "positive" ] [ "non-positive" ] if
```

### `when ( condition lambda -- )`
Conditional execution without else clause.

```tardi
debug-mode [ "Debug information" println ] when
```

### `while ( predicate body -- )`
Loop while predicate returns true.

```tardi
[ counter 10 < ] [ 
    counter println
    counter 1 + >counter 
] while
```

## Stack Effect Documentation

### Standard Notation

Stack effects use the notation `( before -- after )`:

- `before`: Items consumed from stack (bottom to top, left to right)
- `after`: Items produced on stack (bottom to top, left to right)
- `...`: Additional stack items (unchanged)

#### Examples

```tardi
: dup ( a -- a a )           // Duplicates top item
: swap ( a b -- b a )        // Swaps top two items  
: + ( a b -- sum )           // Adds two numbers
: empty? ( collection -- ? ) // Tests if collection is empty
```

### Advanced Stack Effects

```tardi
: dip ( ... x lambda -- ... lambda-result x )
: keep ( ... x lambda -- ... lambda-result x )
: if ( condition true-lambda false-lambda -- result )
```

## Error Handling in Functions

Functions should handle errors gracefully:

```tardi
: safe-divide ( a b -- result success? )
    dup 0 == [
        2drop 0 #f  // Return 0 and failure flag
    ] [
        / #t        // Return result and success flag
    ] if
;

10 0 safe-divide  // Returns: 0 #f
10 2 safe-divide  // Returns: 5 #t
```

## Module Integration

Functions and macros work seamlessly with the module system:

```tardi
// Define functions that use imported modules
uses: std/io
uses: std/hashmaps

: log-to-file ( message filename -- )
    <writer>
    swap over write-line
    close
;

: config-get ( key -- value )
    global-config get drop
;
```

## Best Practices

### Function Design

1. **Single Responsibility**: Each function should do one thing well
2. **Clear Stack Effects**: Always document stack effects
3. **Descriptive Names**: Use names that indicate purpose
4. **Error Handling**: Consider failure cases

```tardi
// Good: Clear purpose and stack effect
: circle-area ( radius -- area )
    dup * 3.14159 *
;

// Better: With error handling
: circle-area ( radius -- area success? )
    dup 0 >= [
        dup * 3.14159 * #t
    ] [
        drop 0 #f
    ] if
;
```

### Macro Design

1. **Use Sparingly**: Macros add complexity
2. **Clear Syntax**: Design intuitive syntax
3. **Documentation**: Document macro behavior clearly
4. **Testing**: Test macro expansion thoroughly

### Code Organization

```tardi
// Group related functions
: string-empty? ( str -- ? ) length 0 == ;
: string-length ( str -- n ) length ;
: string-first ( str -- char ) 0 swap nth ;

// Use descriptive intermediate functions
: is-vowel? ( char -- ? )
    { 'a' 'e' 'i' 'o' 'u' } swap in?
;

: count-vowels ( string -- count )
    0 swap [ is-vowel? [ 1 + ] when ] each
;
```

## Performance Considerations

- Functions have minimal call overhead
- Lambdas are compiled efficiently
- Recursive functions use the return stack
- Tail recursion is supported but not optimized
- Macros have zero runtime cost (compile-time only)