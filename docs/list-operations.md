# List Operations

This document describes the list operations available in the Tardi language.

## List Literals

Lists can be created using curly brace literals:

```
{ 1 2 3 }  // Creates a list [1 2 3]
{ }        // Creates an empty list
{ "hello" 42 #t }  // Lists can contain mixed types
```

List literals are implemented using the macro system, which expands them into appropriate list creation and manipulation operations.

## Creating a List Programmatically

To create a new empty list, use the `<list>` word:

```
<list>  // Creates an empty list and pushes it onto the stack
```

## List Operations

### append

Appends a value to the end of a list.

Stack effect: `( x list -- )`

```
<list> 42 append  // Creates a list [42]
{ } 42 append    // Same result using list literal
```

### prepend

Prepends a value to the beginning of a list.

Stack effect: `( x list -- )`

```
<list> 1 prepend  // Creates a list [1]
{ } 1 prepend    // Same result using list literal
```

### concat

Concatenates two lists.

Stack effect: `( list1 list2 -- list3 )`

```
{ 1 } { 2 } concat  // Creates a list [1 2]
```

### split-head!

Removes and returns the first element of a list. The modified list remains on the stack.

Stack effect: `( list -- x )`

```
{ 1 2 } split-head!  // Results in 1 on the stack, leaves [2] as the modified list
```

## Error Handling

- Attempting to append or prepend to a non-list value will result in a `TypeMismatch` error.
- Using `split-head!` on an empty list will result in an `EmptyList` error.
- Invalid list literals will result in compilation errors.

## Implementation Details

- Lists are implemented as `Value::List(Vec<SharedValue>)`.
- List operations preserve the shared value system using `Rc<RefCell<Value>>`.
- List literals are implemented through the macro system, which expands them at compile time.
- The current implementation keeps list operations minimal, with plans for future enhancements such as:
  - More advanced list manipulation functions
  - List comprehensions
  - Pattern matching on lists
  - Advanced iteration capabilities

## Examples

```
// Creating and manipulating lists
{ 1 2 3 }                // Creates [1 2 3]
{ } 42 append            // Creates [42]
{ 1 } { 2 } concat      // Creates [1 2]

// Mixed type lists
{ "hello" 42 #t }       // Creates ["hello" 42 #t]

// Nested lists
{ { 1 2 } { 3 4 } }    // Creates [[1 2] [3 4]]

// List manipulation
{ 1 2 3 } split-head!  // Returns 1, leaves [2 3]
