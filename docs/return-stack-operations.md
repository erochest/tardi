# Return Stack Operations

The return stack is a secondary stack used for temporary storage and control flow. It operates independently from the main data stack.

## Operations

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

## Error Handling

- ReturnStackOverflow: Occurs when attempting to push to a full return stack (1024 items)
- ReturnStackUnderflow: Occurs when attempting to pop or peek from an empty return stack

## Usage Notes

- The return stack is primarily used for temporary storage and control flow operations
- Each stack (data and return) has a maximum capacity of 1024 items
- Values on the return stack maintain their types (Integer, Float, Boolean)
- Operations preserve the shared value system using Rc<RefCell<Value>>
