# Control Flow in Tardi

Tardi provides several control flow constructs for conditional execution, loops, and program flow control. All control flow operations work with the stack-based execution model and support lambda expressions for code blocks.

## Conditional Execution

### `if ( condition true-lambda false-lambda -- result )`

The primary conditional construct. Executes one of two lambdas based on a boolean condition.

```tardi
// Basic if statement
5 0 > [ "positive" ] [ "not positive" ] if
// Results in: "positive"

// Numeric computation
x 0 < [ x -1 * ] [ x ] if  // Absolute value

// Complex conditional logic
score 90 >= 
    [ "Grade A" ] 
    [ score 80 >= [ "Grade B" ] [ "Grade C" ] if ] 
if
```

#### Stack Effects

- Consumes: condition (boolean), true-lambda, false-lambda
- Produces: Result from executed lambda

### `when ( condition lambda -- )`

Conditional execution without an else clause. Only executes the lambda if the condition is true.

```tardi
// Simple conditional action
debug-mode [ "Debug mode enabled" println ] when

// Guard clause pattern
value 0 < [ "Error: negative value" println return ] when

// Multiple conditions
temperature 100 > [ "Water is boiling!" println ] when
temperature 0 < [ "Water is frozen!" println ] when
```

#### Stack Effects

- Consumes: condition (boolean), lambda
- Produces: Nothing (or lambda result if executed)

## Comparison Operations

These operations produce boolean values for use with conditionals:

### `== ( a b -- boolean )`

Tests equality between two values.

```tardi
5 5 ==     // #t
"a" "b" == // #f
```

### `< ( a b -- boolean )`

Tests if first value is less than second.

```tardi
3 5 <      // #t
10 2 <     // #f
```

### `> ( a b -- boolean )`

Tests if first value is greater than second.

```tardi
7 3 >      // #t
1 5 >      // #f
```

### `! ( boolean -- inverted-boolean )`

Logical negation.

```tardi
#t !       // #f
#f !       // #t
5 0 == !   // #t (5 is not equal to 0)
```

## Loops

### `while ( predicate body -- )`

Executes body repeatedly while predicate returns true. Both predicate and body are lambdas.

TODO: make clear that nothing is stored between predicate and body. you need to dup yourself
TODO: while can also use `break` and `continue` words

```tardi
// Simple counter loop
0 >counter
[ counter 10 < ] [
    counter println
    counter 1 + >counter
] while

// Processing with termination condition
[ list empty? ! ] [
    list pop! process-item
] while

// Input processing loop
[ 
    "Enter command (quit to exit): " print
    <stdin> read-line
    dup "quit" == !
] [
    process-command
] while
drop  // Clean up final input
```

#### Implementation Details

- Predicate is tested before each iteration
- Both predicate and body lambdas have access to the current stack
- Loop exits when predicate returns `#f`

### `loop` with `break` and `continue`

Advanced loop construct with explicit control flow.

```tardi
[
    // Loop body
    get-next-item
    
    // Skip processing for certain items
    dup skip-item? [ drop continue ] when
    
    // Exit loop on termination condition
    dup end-item? [ drop break ] when
    
    // Process the item
    process-item
] loop
```

#### `break ( -- )`

Exits the current loop immediately.

```tardi
[
    user-input
    dup "exit" == [ break ] when
    process-input
] loop
```

#### `continue ( -- )`

Skips to the next iteration of the current loop.

TODO: std/math/zero?

```tardi
[
    get-number
    dup 0 == [ continue ] when  // Skip zeros
    dup negative? [ continue ] when  // Skip negatives
    process-positive-number
] loop
```

## Advanced Control Flow Patterns

### Nested Conditionals

```tardi
: classify-number ( n -- description )
    dup 0 == [ drop "zero" ] [
        dup 0 > [
            dup 1 == [ drop "one" ] [
                dup even? [ "positive even" ] [ "positive odd" ] if
            ] if
        ] [
            "negative"
        ] if
    ] if
;
```

### Early Return Pattern

TODO: `return` leaves the immediate lambda, not the primary one. is there a way to break from the named function? have return do that?

```tardi
: validate-input ( input -- result )
    // Check for null/empty
    dup empty? [ drop "Error: empty input" return ] when
    
    // Check for valid format
    dup valid-format? ! [ drop "Error: invalid format" return ] when
    
    // Check for reasonable length
    dup length 100 > [ drop "Error: too long" return ] when
    
    // All checks passed
    "Valid input"
;
```

### State Machine Pattern

TODO: this isn't a valide `while`

```tardi
: state-machine ( initial-state -- final-state )
    [
        dup "idle" == [
            handle-idle-state
            dup "exit" == [ break ] when
        ] when
        
        dup "processing" == [
            handle-processing-state
        ] when
        
        dup "error" == [
            handle-error-state
        ] when
        
        // Continue until exit state
        dup "exit" == !
    ] while
;
```

### Exception-Style Error Handling

```tardi
: safe-operation ( input -- result success? )
    // Try the operation
    dup risky-operation
    
    // Check for success
    dup valid-result? [
        #t  // Success flag
    ] [
        drop "Error occurred" #f  // Error result and failure flag
    ] if
;

: main-operation ( input -- )
    safe-operation [
        // Success path
        "Operation completed: " swap concat println
    ] [
        // Error path
        "Operation failed: " swap concat println
    ] if
;
```

## Control Flow with Data Structures

### Processing Lists

TODO: rest should probably be pop!

```tardi
: process-all-items ( list -- )
    [
        dup empty? !  // Continue while not empty
    ] [
        dup first     // Get first item
        process-item  // Process it
        rest          // Remove first item
    ] while
    drop  // Clean up empty list
;
```

### Hash Map Iteration with Control Flow

TODO: `over over` to `2dup`. and in other files
TODO: i have doubts about the stack manipulation in this

```tardi
uses: std/hashmaps

: find-in-hashmap ( target hashmap -- value found? )
    #f swap  // Initialize found flag
    [
        // For each key-value pair
        over over get [  // Get value for current key
            rot 2dup == [  // Check if value matches target
                drop #t swap  // Set found flag, keep value
                break         // Exit early
            ] [
                drop         // Not a match, continue
            ] if
        ] [
            drop  // Key not found, continue
        ] if
    ] each
;
```

## Performance Considerations

### Conditional Optimization

TODO: `unless`

```tardi
// Efficient: short-circuit evaluation
: fast-check ( x -- result )
    dup cheap-test [
        expensive-operation
    ] [
        default-value
    ] if
;

// Less efficient: always evaluates both
: slow-check ( x -- result )
    dup expensive-operation swap cheap-test
    [ ] [ drop default-value ] if
;
```

### Loop Optimization

TODO: or do i want to just add `rest`?

```tardi
// Efficient: minimize work in loop condition
: efficient-loop ( list -- )
    dup length >count
    0 >index
    [ index count < ] [
        dup index nth process-item
        index 1 + >index
    ] while
    drop
;

// Less efficient: recalculates length each iteration
: inefficient-loop ( list -- )
    [ dup length 0 > ] [
        dup first process-item
        rest
    ] while
    drop
;
```

## Best Practices

### 1. Use Appropriate Control Flow

```tardi
// Good: Use when for single-condition actions
error-occurred [ log-error ] when

// Good: Use if for binary choices
user-admin? [ admin-menu ] [ user-menu ] if

// Good: Use while for condition-based loops
[ more-data? ] [ process-data ] while
```

### 2. Keep Lambdas Simple

```tardi
// Good: Simple, focused lambdas
valid-input? [ process-input ] [ show-error ] if

// Better: Extract complex logic to functions
: handle-valid-input ( input -- )
    validate-further
    transform-data
    save-result
;

valid-input? [ handle-valid-input ] [ show-error ] if
```

### 3. Handle Edge Cases

```tardi
: safe-divide ( a b -- result )
    dup 0 == [
        2drop 0  // Handle division by zero
    ] [
        /        // Normal division
    ] if
;
```

### 4. Document Complex Control Flow

```tardi
// Process items until we find a match or exhaust the list
: find-match ( target list -- item found? )
    [
        dup empty? !  // Continue while list not empty
    ] [
        dup first over ==  // Check if first item matches target
        [ #t break ] [     // Found match, exit loop
            rest           // Not a match, try rest of list
        ] if
    ] while
    
    // Clean up and return result
    dup empty? [
        2drop #f  // List exhausted, no match found
    ] [
        first #t  // Return matching item
    ] if
;
```

