# Hash Map Operations in Tardi

Hash maps in Tardi provide efficient key-value storage with O(1) average lookup time. They support immutable keys and mutable values, making them ideal for dictionaries, caches, and configuration storage.

## Module Import

To use hash map operations, import the hashmaps module:

```tardi
uses: std/hashmaps
```

## Hash Map Literals

### `H{ ... }` - Hash Map Literal Syntax

Hash maps use the `H{` `}` literal syntax with key-value pairs represented as 2-element vectors:

```tardi
H{ { "name" "Alice" } { "age" 30 } { "city" "New York" } }
```

Each key-value pair is a vector containing exactly two elements: `{ key value }`.

## Creating Hash Maps

### `<hashmap> ( -- hashmap )`

Creates an empty hash map.

```tardi
<hashmap>  // Creates: H{ }
```

### `>hashmap ( vector-of-pairs -- hashmap )`

Converts a vector of key-value pairs into a hash map.

```tardi
{ { "a" 1 } { "b" 2 } { "c" 3 } } >hashmap
// Creates: H{ { "a" 1 } { "b" 2 } { "c" 3 } }
```

## Converting Hash Maps

### `>vector ( hashmap -- vector-of-pairs )`

Converts a hash map back to a vector of key-value pairs.

```tardi
H{ { "x" 10 } { "y" 20 } } >vector
// Returns: { { "x" 10 } { "y" 20 } }
```

## Query Operations

### `is-hashmap? ( object -- boolean )`

Tests whether an object is a hash map.

```tardi
H{ { "test" 42 } } is-hashmap?  // Returns: #t
{ 1 2 3 } is-hashmap?           // Returns: #f
42 is-hashmap?                  // Returns: #f
```

### `length ( hashmap -- count )`

Returns the number of key-value pairs in the hash map.

```tardi
H{ { "a" 1 } { "b" 2 } } length  // Returns: 2
<hashmap> length                 // Returns: 0
```

### `empty? ( hashmap -- boolean )`

Tests whether a hash map is empty.

```tardi
<hashmap> empty?                      // Returns: #t
H{ { "key" "value" } } empty?         // Returns: #f
```

### `get ( key hashmap -- value found? )`

Retrieves a value by key, returning both the value and a success flag.

```tardi
"name" H{ { "name" "Bob" } { "age" 25 } } get
// Returns: "Bob" #t

"missing" H{ { "name" "Bob" } } get
// Returns: #f #f
```

### `in? ( key hashmap -- boolean )`

Tests whether a key exists in the hash map.

```tardi
"name" H{ { "name" "Carol" } { "age" 35 } } in?  // Returns: #t
"city" H{ { "name" "Carol" } { "age" 35 } } in?  // Returns: #f
```

## Accessing Keys and Values

### `keys ( hashmap -- vector-of-keys )`

Returns a vector containing all keys from the hash map.

```tardi
H{ { "name" "Dave" } { "age" 40 } { "city" "Boston" } } keys
// Returns: { "name" "age" "city" } (order may vary)
```

### `values ( hashmap -- vector-of-values )`

Returns a vector containing all values from the hash map.

```tardi
H{ { "a" 100 } { "b" 200 } { "c" 300 } } values
// Returns: { 100 200 300 } (order may vary)
```

## Modification Operations

### `set! ( key value hashmap -- )`

Sets a key-value pair in the hash map (modifies in place).

```tardi
H{ { "age" 28 } } "name" "Eve" pick set!
// Results in: H{ { "age" 28 } { "name" "Eve" } }

H{ { "name" "Eve" } { "age" 28 } } "age" 29 pick  set!
// Updates existing key: H{ { "name" "Eve" } { "age" 29 } }
```

### `add! ( pair-vector hashmap -- )`

Adds a key-value pair from a 2-element vector.

```tardi
H{ { "name" "Eve" } } { "email" "eve@example.com" } over add!
// Results in: H{ { "name" "Eve" } { "email" "eve@example.com" } }
```

## Iteration Operations

### `each ( hashmap lambda -- )`

Applies a function to each key-value pair. The lambda receives the key and value as separate arguments.

```tardi
: print-pair ( key value -- )
    swap print ": " print println ;

H{ { "name" "Frank" } { "age" 45 } { "city" "Seattle" } }
[ print-pair ] each
// Output:
// name: Frank
// age: 45
// city: Seattle
```

### `map ( hashmap lambda -- hashmap' )`

Creates a new hash map by applying a function to each value. The lambda receives the value and returns a new value.

```tardi
H{ { "a" 1 } { "b" 2 } { "c" 3 } }
[ 10 * ] map
// Returns: H{ { "a" 10 } { "b" 20 } { "c" 30 } }
```

## Type Compatibility

Hash map keys can be any type that supports equality comparison:

```tardi
// String keys
H{ { "name" "Alice" } { "status" "active" } }

// Integer keys  
H{ { 1 "first" } { 2 "second" } { 3 "third" } }

// Mixed key types
H{ { "count" 42 } { 100 "century" } { #t "boolean key" } }

// Character keys
H{ { 'a' 1 } { 'b' 2 } { 'c' 3 } }
```

Values can be any type:

```tardi
H{ 
    { "number" 42 }
    { "string" "hello" }
    { "boolean" #t }
    { "list" { 1 2 3 } }
    { "function" [ dup * ] }
}
```

## Common Patterns

### Building Hash Maps Incrementally

```tardi
: build-config ( -- hashmap )
    <hashmap>
    "debug" #t over set!
    "port" 8080 over set!
    "host" "localhost" over set!
;

build-config
```

### Merging Hash Maps

```tardi
: merge-hashmaps ( hashmap1 hashmap2 -- merged-hashmap )
    swap >vector  // Convert first hashmap to vector
    [ over add! ] each  // Add each pair to second hashmap
;

H{ { "a" 1 } { "b" 2 } }
H{ { "c" 3 } { "d" 4 } }
merge-hashmaps
// Results in: H{ { "a" 1 } { "b" 2 } { "c" 3 } { "d" 4 } }
```

### Configuration Management

```tardi
: load-config ( -- config )
    H{
        { "database_url" "postgresql://localhost/app" }
        { "debug_mode" #f }
        { "max_connections" 100 }
        { "cache_timeout" 300 }
    } ;

: get-config ( key -- value )
    load-config get drop ;

"debug_mode" get-config  // Returns: #f
```

### Counting Occurrences

```tardi
: count-chars ( string -- hashmap )
    <hashmap> swap
    [ 
        // For each character
        over over get  // Get current count
        [ 1 + ] [ 1 ] if  // Increment or start at 1
        rot set!  // Update the count
    ] each ;

"hello world" count-chars
// Returns hash map with character frequencies
```

### Data Transformation

```tardi
: process-users ( users -- processed )
    [  // For each user record
        "name" over get drop >uppercase  // Get and uppercase name
        "name" rot set!  // Update name in record
    ] map ;

{ 
    H{ { "name" "alice" } { "age" 25 } }
    H{ { "name" "bob" } { "age" 30 } }
} 
[ process-users ] each
```

### Caching Results

TODO: `constant:`

```tardi
<hashmap> constant: cache

: cached-fibonacci ( n -- result )
    dup cache get  // Check if already computed
    [  // If found in cache
        nip  // Use cached result
    ] [  // If not in cache
        dup fibonacci  // Compute fibonacci
        dup rot cache set! drop  // Store in cache
    ] if ;
```

## Performance Notes

- Hash maps provide O(1) average-case lookup, insertion, and deletion
- Keys are immutable after creation (frozen keys)
- Iteration order is not guaranteed to be consistent
- Hash maps are mutable structures - operations like `set!` and `add!` modify in place, but consume it's place in the stack, so be sure to `dup` the hash map in some way.
- Key comparison uses Tardi's built-in equality semantics

## Error Handling

- `get` returns a boolean flag to indicate success/failure
- Missing keys in `get` return `#f` twice
- `in?` provides a direct boolean test for key existence
- Invalid key-value pair vectors (not exactly 2 elements) may cause runtime errors

