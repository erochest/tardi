# Stack Manipulation

## Overview

Tardi is a stack-based language where operations primarily work by manipulating values on a data stack. Values are pushed onto the stack, and operations consume values from the stack and may push results back.

## Stack Effect Notation

Stack effects are written in the form ( before -- after ) where:

- Values before the -- are what the operation consumes from the stack (top of stack rightmost)
- Values after the -- are what the operation pushes to the stack (top of stack rightmost)

For example: swap ( a b -- b a ) means the operation takes two values from the stack and pushes them back in reverse order.

## Basic Stack Operations

### `dup ( a -- a a )`

Duplicates the top item on the stack.

Example:

```tardi
42 dup    // Stack: 42 42
```

### `swap ( a b -- b a )`

Exchanges the top two items on the stack.

Example:

```tardi
1 2 swap  // Stack: 2 1
```

### `rot ( a b c -- b c a )`

Rotates the top three items on the stack, moving the third item to the top.

Example:

```tardi
1 2 3 rot  // Stack: 2 3 1
```

### `drop ( a -- )`

Removes the top item from the stack.

Example:

```tardi
1 2 drop  // Stack: 1
```

### `clear ( ... -- )`

Removes all items from the stack.

Example:

```tardi
1 2 3 clear  // Stack: empty
```

### `stack-size ( -- n )`

Pushes the current size of the stack onto the stack.

Example:

```tardi
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

TODO: let's not have such low-level details

The VM maintains the data stack as a Vec<SharedValue> where SharedValue is Rc<RefCell<Value>>.

## Advanced Stack Combinators

Tardi provides an extensive set of stack manipulation operations beyond the basic ones. These are defined in the bootstrap system and provide powerful data flow control.

### Deep Stack Manipulation

#### `dip ( ...s x lambda -- ...s' x )`

Executes lambda on the stack below the top element.

```tardi
1 2 3 [ + ] dip  // Results in: 3 3 (adds 1+2, preserves 3)
```

#### `2dip ( ...s x y lambda -- ...s' x y )`

Executes lambda on the stack below the top two elements.

```tardi
1 2 3 4 [ + ] 2dip  // Results in: 3 3 4 (adds 1+2, preserves 3 4)
```

#### `3dip ( ...s x y z lambda -- ...s' x y z )`

Executes lambda on the stack below the top three elements.

```tardi
1 2 3 4 5 [ + ] 3dip  // Results in: 3 3 4 5 (adds 1+2, preserves 3 4 5)
```

### Multiple Drop Operations

#### `2drop ( x y -- )`

Removes the top two items from the stack.

```tardi
1 2 3 2drop  // Stack: 1
```

#### `3drop ( x y z -- )`

Removes the top three items from the stack.

```tardi
1 2 3 4 3drop  // Stack: 1
```

#### `4drop ( w x y z -- )`

Removes the top four items from the stack.

#### `5drop ( v w x y z -- )`

Removes the top five items from the stack.

### Nip Operations (Drop Below Top)

#### `nip ( x y -- y )`

Removes the second item from the stack.

```tardi
1 2 nip  // Stack: 2
```

#### `2nip ( x y z -- z )`

Removes the second and third items from the stack.

#### `3nip ( w x y z -- z )`

Removes the second, third, and fourth items from the stack.

#### `4nip ( v w x y z -- z )`

Removes the second through fifth items from the stack.

#### `5nip ( u v w x y z -- z )`

Removes the second through sixth items from the stack.

### Deep Duplication

#### `dupd ( x y -- x x y )`

Duplicates the second item on the stack.

```tardi
1 2 dupd  // Stack: 1 1 2
```

#### `2dup ( x y -- x y x y )`

Duplicates the top two items on the stack.

```tardi
1 2 2dup  // Stack: 1 2 1 2
```

#### `3dup ( x y z -- x y z x y z )`

Duplicates the top three items on the stack.

```tardi
1 2 3 3dup  // Stack: 1 2 3 1 2 3
```

### Deep Swapping

#### `swapd ( x y z -- y x z )`

Swaps the second and third items on the stack.

```tardi
1 2 3 swapd  // Stack: 2 1 3
```

#### `2swap ( w x y z -- y z w x )`

Swaps the top two pairs of items.

```tardi
1 2 3 4 2swap  // Stack: 3 4 1 2
```

### Over Operations (Copy From Below)

#### `over ( x y -- x y x )`

Copies the second item to the top of the stack.

```tardi
1 2 over  // Stack: 1 2 1
```

#### `overd ( x y z -- x y x z )`

Copies the third item over the second item.

#### `2over ( x y z -- x y z x y )`

Copies the third and fourth items to the top.

### Rotation Operations

#### `-rot ( x y z -- z x y )`

Rotates the top three items in reverse direction.

```tardi
1 2 3 -rot  // Stack: 3 1 2
```

#### `spin ( x y z -- z y x )`

Spins the top three items.

```tardi
1 2 3 spin  // Stack: 3 2 1
```

#### `4spin ( w x y z -- z y x w )`

Spins the top four items.

### Pick and Reach

#### `pick ( x y z -- x y z x )`

Copies the third item to the top.

```tardi
1 2 3 pick  // Stack: 1 2 3 1
```

#### `reach ( w x y z -- w x y z w )`

Copies the fourth item to the top.

### Preserving Combinators

#### `keep ( ...x lambda -- ...x' x )`

Applies lambda to the stack but preserves the top value.

```tardi
10 [ 2 * ] keep  // Stack: 20 10
```

#### `2keep ( ...a x y lambda -- ...b x y )`

Applies lambda but preserves the top two values.

#### `3keep ( ...a x y z lambda -- ...b x y z )`

Applies lambda but preserves the top three values.

## Examples with Different Types

```tardi
// Numbers
42 dup         // Stack: 42 42
3.14 2 swap    // Stack: 2 3.14

// Booleans
#t #f swap     // Stack: #f #t

// Strings
"hello" dup    // Stack: "hello" "hello"

// Vectors
{ 1 2 } dup    // Stack: { 1 2 } { 1 2 }

// Mixed types
42 "hello" #t rot  // Stack: "hello" #t 42
```

## Function Composition

Stack manipulation operations are essential for function composition and data flow control:

```tardi
// Example function using stack manipulation
: example ( a b c -- b c a )
    rot    // Rotate third item to top
;

// Using preserving combinators
: calculate-stats ( numbers -- sum count avg )
    dup [ + ] reduce      // Calculate sum, keep original
    swap dup length       // Get count
    [ swap / ] dip        // Calculate average
;

// Complex stack manipulation with keep
: process-data ( data -- processed-data original-data )
    [ transform-function ] keep
;

// Using dip for intermediate calculations
: complex-calculation ( a b c -- result )
    [ + ] dip    // Add a+b, keep c on top
    *            // Multiply sum by c
;
```

## Best Practices

1. Use stack manipulation operations judiciously to maintain code clarity
2. Consider using the return stack for temporary storage in complex operations
3. Document stack effects clearly for all function definitions
4. Use clear names that indicate the stack effect when defining new words
5. Keep stack manipulations simple and composable
6. Consider factoring complex stack manipulations into named operations
