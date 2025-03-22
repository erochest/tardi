# List Operations

This document describes the list operations available in the Tardi language.

## Creating a List

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
```

### prepend

Prepends a value to the beginning of a list.

Stack effect: `( x list -- )`

```
<list> 1 prepend  // Creates a list [1]
```

### concat

Concatenates two lists.

Stack effect: `( list1 list2 -- list3 )`

```
<list> 1 append <list> 2 append concat  // Creates a list [1 2]
```

### split-head!

Removes and returns the first element of a list. The modified list remains on the stack.

Stack effect: `( list -- x )`

```
<list> 1 append 2 append split-head!  // Results in 1 on the stack
```

## Error Handling

- Attempting to append or prepend to a non-list value will result in a `TypeMismatch` error.
- Using `split-head!` on an empty list will result in an `EmptyList` error.

## Implementation Details

- Lists are implemented as `Value::List(Vec<SharedValue>)`.
- List operations preserve the shared value system using `Rc<RefCell<Value>>`.
- The current implementation keeps list operations minimal, with plans for future enhancements such as more advanced list manipulation functions.
