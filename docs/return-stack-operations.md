# Return Stack Operations

The return stack is a secondary stack used for temporary storage, control flow, and function call management. It operates independently from the main data stack.

## Basic Operations

### >r (to-r)

Moves a value from the data stack to the return stack.

Example:

```forth
42 >r  // Moves 42 from data stack to return stack
```

### r> (r-from)

Moves a value from the return stack to the data stack.

Example:

```forth
42 >r  // Move 42 to return stack
r>     // Move 42 back to data stack
```

### r@ (r-fetch)

Copies the top value from the return stack to the data stack without removing it from the return stack.

Example:

```forth
42 >r  // Move 42 to return stack
r@     // Copy 42 to data stack, leaving it on return stack
r>     // Move 42 from return stack to data stack
```

## Function Calls and Control Flow

The return stack plays a crucial role in managing function calls and control flow:

### Function Calls

When a function is called, the return address (the location to continue execution after the function completes) is automatically pushed onto the return stack.

### Return Operation

The `return` operation uses the top value of the return stack to determine where to continue execution after a function call.

### Recursion

The return stack enables recursive function calls by storing multiple return addresses.

## Error Handling

- ReturnStackOverflow: Occurs when attempting to push to a full return stack (1024 items)
- ReturnStackUnderflow: Occurs when attempting to pop or peek from an empty return stack

## Usage Notes

- The return stack is primarily used for temporary storage, control flow operations, and function call management
- Each stack (data and return) has a maximum capacity of 1024 items
- Values on the return stack maintain their types (Integer, Float, Boolean, Address)
- Operations preserve the shared value system using Rc<RefCell<Value>>
- Manual manipulation of the return stack (using >r, r>, r@) should be done with caution, especially within functions, to avoid interfering with the normal function call mechanism

## Advanced Usage

### Temporary Storage in Functions

The return stack can be used for temporary storage within functions, allowing for more efficient use of the data stack:

```forth
: example ( a b c -- result )
    >r >r  // Store b and c on the return stack
    2 *    // Operate on a
    r> *   // Retrieve and use c
    r> +   // Retrieve and use b
;
```

TODO: mention that combinators like `keep` and `dip` are better alternatives that abstract away use of the return stack

### Implementing Control Structures

The return stack is used internally to implement control structures like loops and conditionals. While these are typically abstracted away from the user, understanding the return stack's role can be helpful for advanced programming techniques.

## Best Practices

1. Always ensure that any values pushed to the return stack are popped off before the end of a function.
2. Be cautious when manipulating the return stack manually within functions to avoid corrupting the call stack.
3. Use the return stack for its intended purposes: temporary storage, control flow, and function calls. Avoid using it as a secondary data stack for long-term storage.
4. When implementing complex control structures or low-level optimizations, consider the impact on the return stack and ensure proper stack balance is maintained.
