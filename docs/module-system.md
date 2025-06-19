# Module System in Tardi

Tardi's module system enables code organization, namespace management, and library distribution. It provides explicit import/export controls and supports both standard library modules and user-defined modules.

## Module Structure

Every Tardi module consists of:

1. **Import declarations** (`uses:`) - What the module depends on
2. **Export declarations** (`exports:`) - What the module provides
3. **Implementation** - Functions, macros, and constants

## Import System

### `uses:` - Importing Modules

The `uses:` macro imports functionality from other modules:

```tardi
uses: module-name
```

Multiple modules can be imported:

```tardi
uses: std/io
uses: std/hashmaps  
uses: std/vectors
```

#### Standard Library Modules

The standard library provides these modules:

- `std/io` - File and console I/O operations
- `std/hashmaps` - Hash map data structures and operations
- `std/vectors` - Vector/list operations and utilities
- `std/fs` - File system operations
- `std/strings` - String manipulation functions
- `std/_internals` - Internal VM functions
- `std/scanning` - Source code scanning utilities
- `std/kernel` - Core VM operations (automatically available)

#### Module Dependencies

Modules automatically import their dependencies:

```tardi
uses: std/hashmaps  // Automatically imports std/vectors
uses: std/io        // Standalone module
```

TODO: cover name resolution and name conflicts
TODO: another doc page for configuration

## Export System

### `exports:` - Defining Module Interface

The `exports:` macro defines what functions and symbols a module makes available:

```tardi
exports: function1 function2 MACRO1 constant1 ;
```

#### Example Module Structure

```tardi
// math-utils.tardi
uses: std/kernel

exports: square cube is-even? is-odd? factorial ;

: square ( n -- n^2 )
    dup *
;

: cube ( n -- n^3 )
    dup square *
;

: is-even? ( n -- ? )
    2 % 0 ==
;

: is-odd? ( n -- ? )
    is-even? !
;

: factorial ( n -- n! )
    dup 1 <= [ drop 1 ] [
        dup 1 - factorial *
    ] if
;
```

## Standard Library Overview

### Core Module (`std/kernel`)

Automatically available in all programs. Provides:

- Stack operations: `dup`, `swap`, `rot`, `drop`, `clear`
- Arithmetic: `+`, `-`, `*`, `/`
- Comparisons: `==`, `<`, `>`, `!`
- Control flow: `if`, `when`, `while`, `loop`
- Function operations: `apply`, `return`, `call`

```tardi
// These are always available
42 dup *     // 1764
5 3 >        // #t
```

### I/O Module (`std/io`)

```tardi
uses: std/io

"Hello, world!" println
"/tmp/test.txt" "File content" write-file
"/tmp/test.txt" read-file println drop
```

### Hash Maps Module (`std/hashmaps`)

```tardi
uses: std/hashmaps

H{ { "name" "Alice" } { "age" 30 } }
"name" over get println drop  // Prints: Alice
```

### Vectors Module (`std/vectors`)

```tardi
uses: std/vectors

{ 1 2 3 4 5 }
[ dup * ] map     // { 1 4 9 16 25 }
[ 2 % 0 == ] filter  // Keep even numbers
```

### File System Module (`std/fs`)

```tardi
uses: std/fs

"/tmp/testdir" ensure-dir
"/tmp/testfile.txt" touch
"/tmp/testdir" ls println
```

### Strings Module (`std/strings`)

```tardi
uses: std/strings

"Hello, World!" >lowercase     // "hello, world!"
"foo,bar,baz" "," split-all    // { "foo" "bar" "baz" }
"   trimmed   " strip-whitespace
```

## Creating Custom Modules

### File-Based Modules

Create a `.tardi` file with module structure:

```tardi
// geometry.tardi
uses: std/kernel

exports: circle-area rectangle-area triangle-area PI ;

3.14159265359 constant: PI

: circle-area ( radius -- area )
    dup * PI *
;

: rectangle-area ( width height -- area )
    *
;

: triangle-area ( base height -- area )
    * 2 /
;
```

### Using Custom Modules

```tardi
uses: geometry

5 circle-area println    // Calculate and print circle area
10 20 rectangle-area println  // Calculate rectangle area
```

## Module Resolution

Tardi searches for modules in:

1. Current directory
2. Standard library directory (`std/`)
3. User's data directory (`~/.local/share/tardi/std/` on Unix)

### Module File Names

- Module `foo` loads from `foo.tardi`
- Module `std/io` loads from `std/io.tardi`
- Module `mylib/utils` loads from `mylib/utils.tardi`

## Advanced Module Patterns

### Re-exporting from Dependencies

```tardi
// utilities.tardi
uses: std/io
uses: std/hashmaps
uses: std/vectors

// Re-export selected functions
exports: println write-file H{ <vector> push! ;

// Add new functionality
exports: debug-print save-config load-config ;

: debug-print ( obj -- )
    "DEBUG: " print println
;

: save-config ( config filename -- )
    <writer>
    swap >vector
    [ [ first ] keep second
      over print ": " print println
    ] each
    close
;
```

### Conditional Exports

TODO: I'm not sure about this one

```tardi
// platform-utils.tardi
uses: std/kernel
uses: std/fs

exports: get-temp-dir path-separator ;

// Platform-specific implementations
: get-temp-dir ( -- path )
    // Implementation varies by platform
    "/tmp"  // Unix default
;

: path-separator ( -- separator )
    "/"  // Unix separator
;
```

### Module Initialization

TODO: constants

```tardi
// logger.tardi
uses: std/io
uses: std/hashmaps

exports: log-info log-error log-debug set-log-level ;

// Module-level initialization
H{ { "level" "info" } } constant: log-config

: set-log-level ( level -- )
    "level" log-config set! drop
;

: should-log? ( level -- ? )
    // Implementation for log level checking
    drop #t  // Simplified
;

: log-info ( message -- )
    "info" should-log? [
        "INFO: " swap concat println
    ] [ drop ] if
;
```

## Best Practices

### Module Design

1. **Single Purpose**: Each module should have a focused responsibility
2. **Minimal Interface**: Export only what users need
3. **Clear Dependencies**: Declare all dependencies explicitly
4. **Documentation**: Document module purpose and usage

### Naming Conventions

- Use descriptive module names: `string-utils`, `file-parser`
- Prefix internal functions with underscore: `_internal-helper`
- Use consistent naming across related modules

### Error Handling

```tardi
// safe-math.tardi
uses: std/kernel

exports: safe-divide safe-sqrt ;

: safe-divide ( a b -- result success? )
    dup 0 == [
        2drop 0 #f
    ] [
        / #t
    ] if
;

: safe-sqrt ( n -- result success? )
    dup 0 < [
        drop 0 #f
    ] [
        sqrt #t
    ] if
;
```

### Testing Modules

```tardi
// tests/geometry-test.tardi
uses: geometry

: test-circle-area
    5 circle-area 78.539816 - abs 0.0001 <
    [ "PASS: circle-area" ] [ "FAIL: circle-area" ] if
    println
;

: run-tests
    test-circle-area
    // More tests...
;

run-tests
```

## Module System Implementation

### Import Process

1. **Parse Import**: `uses: module-name` scans module name
2. **Resolve Path**: Find module file in search paths
3. **Load Module**: Parse and compile module if not cached
4. **Import Symbols**: Add exported symbols to current namespace
5. **Handle Dependencies**: Recursively import module dependencies

### Export Process

1. **Parse Exports**: `exports: symbol1 symbol2 ;` collects symbol list
2. **Validate Symbols**: Ensure all exported symbols are defined
3. **Create Interface**: Build module interface for importers
4. **Symbol Resolution**: Make symbols available to importing modules

### Circular Dependencies

Tardi detects and prevents circular dependencies:

```tardi
// module-a.tardi
uses: module-b  // Error if module-b uses module-a

// module-b.tardi  
uses: module-a  // This creates a circular dependency
```

## Performance Considerations

- Modules are compiled once and cached
- Import resolution happens at compile time
- No runtime overhead for module system
- Recursive dependencies are resolved efficiently
- Standard library modules are pre-compiled

## Debugging Module Issues

### Common Problems

1. **Module Not Found**: Check file paths and module names
2. **Circular Dependencies**: Restructure module relationships
3. **Missing Exports**: Ensure all used symbols are exported
4. **Import Order**: Some modules must be imported in specific order

### Debug Techniques

```tardi
// Add debug output to modules
: debug-module-load
    "Loading geometry module" println
;

debug-module-load  // Call at module start
```

