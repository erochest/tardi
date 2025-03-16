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

The VM maintains the data stack as a Vec<Value> where Value can be:
- Integer(i64)
- Float(f64)
- Boolean(bool)

## Future Enhancements

Planned additions to stack manipulation capabilities:
- over ( a b -- a b a )
- 2dup ( a b -- a b a b )
- 2swap ( a b c d -- c d a b )
- nip ( a b -- b )
- tuck ( a b -- b a b )
- pick ( a_n ... a_1 a_0 n -- a_n ... a_1 a_0 a_n )
