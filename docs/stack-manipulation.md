# Stack Manipulation

## Overview

Tardi is a stack-based language where operations primarily work by manipulating values on a data stack. Values are pushed onto the stack, and operations consume values from the stack and may push results back.

## Stack Effect Notation

Stack effects are written in the form ( before -- after ) where:
- Values before the -- are what the operation consumes from the stack (top of stack rightmost)
- Values after the -- are what the operation pushes to the stack (top of stack rightmost)

For example: swap ( a b -- b a ) means the operation takes two values from the stack and pushes them back in reverse order.

## Basic Stack Operations

### dup ( a -- a a )
Duplicates the top item on the stack.

Example:
```
42 dup    // Stack: 42 42
```

### swap ( a b -- b a )
Exchanges the top two items on the stack.

Example:
```
1 2 swap  // Stack: 2 1
```

### rot ( a b c -- b c a )
Rotates the top three items on the stack, moving the third item to the top.

Example:
```
1 2 3 rot  // Stack: 2 3 1
```

### drop ( a -- )
Removes the top item from the stack.

Example:
```
1 2 drop  // Stack: 1
```

### clear ( ... -- )
Removes all items from the stack.

Example:
```
1 2 3 clear  // Stack: empty
```

### stack-size ( -- n )
Pushes the current size of the stack onto the stack.

Example:
```
1 2 3 stack-size  // Stack: 1 2 3 3
```

## Value Types

The stack can hold various types of values:
- Integer(i64)
- Float(f64)
- Boolean(bool)
- Character(char)
- String(String)
- List(Vec<SharedValue>)
- Function(Lambda)

All values on the stack are managed using a shared value system (Rc<RefCell<Value>>) which enables:
- Efficient memory management
- Safe sharing of complex values
- Mutable access when needed

## Error Handling

All stack operations include proper error handling:
- Attempting to pop from an empty stack results in a StackUnderflow error
- Attempting to push to a full stack (>1024 items) results in a StackOverflow error
- Operations requiring multiple items (swap, rot) will fail with StackUnderflow if there aren't enough items

## Implementation Details

Stack operations are implemented across all layers of the system:
- Scanner recognizes operation words as tokens
- Compiler translates tokens into VM instructions
- VM executes the operations using the data stack
- Environment maintains the global state

The VM maintains the data stack as a Vec<SharedValue> where SharedValue is Rc<RefCell<Value>>.

## Examples with Different Types

```
// Numbers
42 dup         // Stack: 42 42
3.14 2 swap    // Stack: 3.14 2

// Booleans
#t #f swap     // Stack: #f #t

// Strings
"hello" dup    // Stack: "hello" "hello"

// Lists
{ 1 2 } dup    // Stack: [1 2] [1 2]

// Mixed types
42 "hello" #t rot  // Stack: "hello" #t 42
```

## Function Composition

Stack manipulation operations are essential for function composition and data flow control:

```
// Example function using stack manipulation
: example ( a b c -- b c a )
    rot    // Rotate third item to top
;

// Using stack operations in functions
: duplicate-top ( a -- a a )
    dup
;

// Complex stack manipulation
: swap-top-two ( a b c -- a c b )
    >r     // Save c to return stack
    swap   // Swap a and b
    r>     // Restore c
;
```

## Best Practices

1. Use stack manipulation operations judiciously to maintain code clarity
2. Consider using the return stack for temporary storage in complex operations
3. Document stack effects clearly for all function definitions
4. Use clear names that indicate the stack effect when defining new words
5. Keep stack manipulations simple and composable
6. Consider factoring complex stack manipulations into named operations
