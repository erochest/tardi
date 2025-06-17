# Vector Operations in Tardi

Vectors in Tardi are dynamic arrays that can hold any type of data. They provide efficient indexed access, modification, and a comprehensive set of operations for data manipulation.

## Module Import

To use advanced vector operations, import the vectors module:

```tardi
uses: std/vectors
```

Basic vector literals and core operations are always available.

## Vector Literals

Vectors are created using curly brace literals:

```tardi
{ 1 2 3 }              // Creates vector [1, 2, 3]
{ }                    // Creates empty vector
{ "hello" 42 #t }      // Vectors can contain mixed types
{ { 1 2 } { 3 4 } }    // Nested vectors
```

Vector literals are implemented using the `{` macro, which expands them into appropriate vector creation operations.

## Creating Vectors Programmatically

### `<vector> ( -- vector )`
Creates a new empty vector.

```tardi
<vector>  // Creates empty vector []
```

## Core Vector Operations

### Modification Operations

#### `push! ( element vector -- vector )`
Adds an element to the end of the vector (mutates in place).

```tardi
{ 1 2 } 3 over push!  // Results in: { 1 2 3 }
```

#### `push-left! ( element vector -- vector )`
Adds an element to the beginning of the vector (mutates in place).

```tardi
{ 2 3 } 1 over push-left!  // Results in: { 1 2 3 }
```

#### `pop! ( vector -- element vector )`
Removes and returns the last element from the vector.

```tardi
{ 1 2 3 } pop!  // Returns: 3 { 1 2 }
```

#### `pop-left! ( vector -- element vector )`
Removes and returns the first element from the vector.

```tardi
{ 1 2 3 } pop-left!  // Returns: 1 { 2 3 }
```

#### `set-nth! ( index value vector -- vector )`
Sets the element at the specified index (mutates in place).

```tardi
{ 1 2 3 } 1 99 over set-nth!  // Results in: { 1 99 3 }
```

#### `sort! ( vector -- vector )`
Sorts the vector in place using natural ordering.

```tardi
{ 3 1 4 1 5 } sort!  // Results in: { 1 1 3 4 5 }
```

### Query Operations

#### `length ( vector -- count )`
Returns the number of elements in the vector.

```tardi
{ 1 2 3 4 5 } length  // Returns: 5
{ } length            // Returns: 0
```

#### `empty? ( vector -- boolean )`
Tests whether the vector is empty.

```tardi
{ } empty?        // Returns: #t
{ 1 2 } empty?    // Returns: #f
```

#### `nth ( index vector -- element )`
Gets the element at the specified index (0-based). Returns `#f` if index is out of bounds.

```tardi
{ 10 20 30 } 1 swap nth  // Returns: 20
{ 10 20 30 } 5 swap nth  // Returns: #f (out of bounds)
```

#### `in? ( element vector -- boolean )`
Tests whether an element exists in the vector.

```tardi
{ 1 2 3 } 2 swap in?    // Returns: #t
{ 1 2 3 } 5 swap in?    // Returns: #f
```

#### `index-of? ( element vector -- index found? )`
Finds the first occurrence of an element, returning index and success flag.

```tardi
{ 10 20 30 20 } 20 swap index-of?  // Returns: 1 #t
{ 10 20 30 } 99 swap index-of?     // Returns: 0 #f
```

### Convenience Accessors (std/vectors)

#### `first ( vector -- element )`
Gets the first element of the vector.

```tardi
{ 10 20 30 } first  // Returns: 10
```

#### `second ( vector -- element )`
Gets the second element of the vector.

```tardi
{ 10 20 30 } second  // Returns: 20
```

#### `third ( vector -- element )`
Gets the third element of the vector.

#### `fourth ( vector -- element )`
Gets the fourth element of the vector.

#### `last ( vector -- element )`
Gets the last element of the vector.

```tardi
{ 10 20 30 } last  // Returns: 30
```

### Structural Operations

#### `concat ( vector1 vector2 -- combined-vector )`
Concatenates two vectors into a new vector.

```tardi
{ 1 2 } { 3 4 } concat  // Returns: { 1 2 3 4 }
```

#### `subvector ( start end vector -- subvector )`
Extracts a portion of the vector from start index (inclusive) to end index (exclusive).

```tardi
{ 1 2 3 4 5 } 1 4 over subvector  // Returns: { 2 3 4 }
```

#### `join ( separator vector -- string )`
Joins vector elements into a string with the specified separator.

```tardi
{ "hello" "world" "!" } " " swap join  // Returns: "hello world !"
{ 1 2 3 } "," swap join               // Returns: "1,2,3"
```

## Higher-Order Functions (std/vectors)

### `each ( vector lambda -- )`
Applies a function to each element of the vector.

```tardi
{ 1 2 3 4 5 } [ dup * println ] each
// Prints: 1, 4, 9, 16, 25
```

### `map ( vector lambda -- new-vector )`
Creates a new vector by applying a function to each element.

```tardi
{ 1 2 3 4 5 } [ dup * ] map  // Returns: { 1 4 9 16 25 }
{ "hello" "world" } [ >uppercase ] map  // Returns: { "HELLO" "WORLD" }
```

### `reduce ( vector initial-value lambda -- result )`
Reduces the vector to a single value using an accumulator function.

```tardi
// Sum all elements
{ 1 2 3 4 5 } 0 [ + ] reduce  // Returns: 15

// Concatenate strings
{ "hello" " " "world" } "" [ concat ] reduce  // Returns: "hello world"

// Find maximum
{ 3 7 2 9 1 } 0 [ [ > ] keep ? ] reduce  // Returns: 9
```

## Advanced Patterns

### Data Transformation Pipelines

```tardi
// Transform, filter, and aggregate data
{ 1 2 3 4 5 6 7 8 9 10 }
[ dup * ]           map    // Square each number: { 1 4 9 16 25 36 49 64 81 100 }
[ 50 > ]           filter  // Keep only values > 50: { 64 81 100 }
0 [ + ]            reduce  // Sum them: 245
```

### Working with Nested Vectors

```tardi
// Matrix operations
{ { 1 2 } { 3 4 } { 5 6 } }
[ [ 2 * ] map ] map  // Double each element: { { 2 4 } { 6 8 } { 10 12 } }

// Flatten nested structure
{ { 1 2 } { 3 4 } { 5 6 } }
{ } [ concat ] reduce  // Returns: { 1 2 3 4 5 6 }
```

### Custom Data Processing

```tardi
uses: std/hashmaps

: process-records ( records -- summary )
    [ 
        // Extract and validate each record
        "status" swap get drop "active" ==
    ] filter
    [
        // Calculate score for active records
        "score" swap get drop 2 *
    ] map
    0 [ + ] reduce  // Sum all scores
;

{
    H{ { "name" "Alice" } { "status" "active" } { "score" 85 } }
    H{ { "name" "Bob" } { "status" "inactive" } { "score" 75 } }
    H{ { "name" "Carol" } { "status" "active" } { "score" 92 } }
}
process-records  // Returns: 354 (85*2 + 92*2)
```

### Vector Building Patterns

```tardi
// Collect results from iteration
: range ( start end -- vector )
    { } -rot
    [ 2dup <= ] [
        pick over push!
        1 +
    ] while
    2drop
;

1 10 range  // Returns: { 1 2 3 4 5 6 7 8 9 10 }

// Generate vector with function
: generate ( count generator -- vector )
    { } -rot
    [ over 0 > ] [
        over 1 - -rot
        dup apply
        rot over push!
        swap
    ] while
    2drop
;

5 [ random ] generate  // Generate 5 random numbers
```

## Error Handling

- Attempting operations on non-vector values results in `TypeMismatch` error
- Index out of bounds returns `#f` for `nth`, may error for `set-nth!`
- Operations on empty vectors (`pop!`, `pop-left!`, `first`, etc.) may error
- Invalid vector literals result in compilation errors

## Performance Considerations

- Mutation operations (`push!`, `pop!`, `set-nth!`) modify vectors in place
- `concat` and `map` create new vectors  
- `each` and `reduce` don't create intermediate vectors
- Indexing with `nth` is O(1) for direct access
- `push!` and `pop!` are typically O(1) operations
- `push-left!` and `pop-left!` are O(n) operations (require shifting elements)

## Implementation Details

- Vectors are implemented as `Value::List(Vec<SharedValue>)`
- All elements use the shared value system (`Rc<RefCell<Value>>`)
- Vector literals use the `{` macro for compile-time expansion
- Standard library functions are implemented in Tardi itself
- Native operations are implemented in Rust for performance

## Common Patterns and Examples

### Data Analysis
```tardi
// Calculate statistics
: stats ( numbers -- min max avg )
    dup [ [ < ] keep ? ] reduce    // Find minimum
    over [ [ > ] keep ? ] reduce   // Find maximum  
    rot dup 0 [ + ] reduce         // Calculate sum
    over length /                  // Calculate average
;

{ 3 7 2 9 1 8 4 } stats  // Returns: 1 9 4.857...
```

### Text Processing
```tardi
uses: std/hashmaps
uses: std/strings

// Word frequency analysis
: word-count ( text -- counts )
    " " split-all
    <hashmap> swap
    [
        over over get [
            1 +
        ] [
            1
        ] if
        rot set!
    ] each
;

"hello world hello tardi world" word-count
```

### Collection Operations
```tardi
// Remove duplicates
: unique ( vector -- unique-vector )
    { } swap
    [
        over over in? ! [
            over push!
        ] [ drop ] if
    ] each
;

{ 1 2 2 3 1 4 3 5 } unique  // Returns: { 1 2 3 4 5 }

// Basic vector operations
{ 1 2 3 }              // Creates [1, 2, 3]
{ } 42 over push!      // Creates [42]
{ 1 2 } { 3 4 } concat // Creates [1, 2, 3, 4]

// Mixed type vectors
{ "hello" 42 #t }      // Creates ["hello", 42, #t]

// Nested vectors
{ { 1 2 } { 3 4 } }    // Creates [[1, 2], [3, 4]]

// Vector manipulation
{ 1 2 3 } pop!         // Returns: 3 [1, 2]
{ 1 2 3 } first        // Returns: 1
```