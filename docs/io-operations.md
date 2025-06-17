# I/O Operations in Tardi

Tardi provides comprehensive input/output capabilities for working with files, console input/output, and standard streams. All I/O operations are provided through the `std/io` module.

## Module Import

To use I/O operations, import the io module:

```tardi
uses: std/io
```

## File Operations

### Writing Files

#### `write-file ( content:string path:string -- success:boolean )`
Writes content to a file, creating the file if it doesn't exist.

```tardi
"Hello, world!" "/tmp/hello.txt" write-file
// Writes "Hello, world!" to /tmp/hello.txt
```

#### File Writers

For more control over file writing, use file writers:

##### `<writer> ( path:string -- writer )`
Creates a writer for the specified file path.

```tardi
"/tmp/output.txt" <writer>  // Creates writer for the file
```

##### `write ( content:string writer -- writer )`
Writes content to a writer.

```tardi
"/tmp/log.txt" <writer> 
"First line" over write
"Second line" over write
close
```

##### `write-line ( content:string writer -- writer )`
Writes content followed by a newline.

```tardi
"/tmp/log.txt" <writer>
"Line 1" over write-line
"Line 2" over write-line
close
```

##### `write-lines ( lines:vector writer -- writer )`
Writes multiple lines from a vector.

```tardi
{ "First line" "Second line" "Third line" } 
"/tmp/multi.txt" <writer>
swap over write-lines
close
```

##### `flush ( writer -- writer )`
Flushes the writer's buffer, ensuring data is written to disk.

```tardi
"/tmp/buffered.txt" <writer>
"Some content" over write
flush  // Force write to disk
close
```

##### `close ( writer -- )`
Closes a writer, automatically flushing any buffered content.

```tardi
"/tmp/example.txt" <writer>
"Content" over write
close  // Automatically flushes and closes
```

### Reading Files

#### `read-file ( path:string -- content:string success:boolean )`
Reads the entire contents of a file as a string.

```tardi
"/tmp/hello.txt" read-file
// Returns: "Hello, world!" #t (if successful)
// Returns: "" #f (if file doesn't exist)
```

#### File Readers

For more control over file reading:

##### `<reader> ( path:string -- reader )`
Creates a reader for the specified file path.

```tardi
"/tmp/input.txt" <reader>  // Creates reader for the file
```

##### `read ( reader -- content:string reader )`
Reads all remaining content from a reader.

```tardi
"/tmp/input.txt" <reader>
read  // Returns entire file content
close
```

##### `read-line ( reader -- line:string reader )`
Reads a single line from a reader.

```tardi
"/tmp/input.txt" <reader>
read-line  // Returns first line
read-line  // Returns second line
close
```

##### `read-lines ( reader -- lines:vector reader )`
Reads all lines into a vector.

```tardi
"/tmp/input.txt" <reader>
read-lines  // Returns vector of all lines
close
```

## Console I/O

### Standard Streams

#### `<stdin> ( -- reader )`
Returns a reader for standard input.

```tardi
<stdin> read-line  // Read a line from console input
drop  // Clean up reader
```

#### `<stdout> ( -- writer )`
Returns a writer for standard output.

```tardi
<stdout> "Hello!" over write close
```

#### `<stderr> ( -- writer )`
Returns a writer for standard error.

```tardi
<stderr> "Error message" over write close
```

### Console Output Functions

#### `print ( object -- )`
Prints an object to standard output without a newline.

```tardi
42 print        // Prints: 42
"hello" print   // Prints: hello
#t print        // Prints: #t
```

#### `println ( object -- )`
Prints an object to standard output followed by a newline.

```tardi
"Hello, world!" println
42 println
```

#### `nl ( -- )`
Prints a newline to standard output.

```tardi
"First part" print nl "Second part" println
// Output:
// First part
// Second part
```

### Error Output Functions

#### `eprint ( object -- )`
Prints an object to standard error without a newline.

```tardi
"Error: " eprint "Something went wrong" eprint
```

#### `eprintln ( object -- )`
Prints an object to standard error followed by a newline.

```tardi
"Fatal error occurred!" eprintln
```

#### `enl ( -- )`
Prints a newline to standard error.

```tardi
"Error prefix" eprint enl
```

## Debug Operations

### `. ( object -- )`
Prints the representation of an object (useful for debugging).

```tardi
42 .           // Prints: 42
{ 1 2 3 } .    // Prints: [1, 2, 3]
"hello" .      // Prints: "hello"
```

### `.s ( -- )`
Prints the contents of the entire stack (non-destructive).

```tardi
1 2 3 "hello" #t
.s
// Prints the stack from bottom to top:
// 1
// 2  
// 3
// "hello"
// #t
```

## Utility Functions

### `file-path>> ( writer-or-reader -- path:string writer-or-reader )`
Gets the file path from a writer or reader.

```tardi
"/tmp/example.txt" <writer>
file-path>>  // Returns: "/tmp/example.txt" writer
```

## Error Handling

Most I/O operations return boolean success flags or handle errors gracefully:

- File operations return `#t` for success, `#f` for failure
- Readers/writers return empty strings or empty vectors on error
- Console operations rarely fail but may block on input

## Common Patterns

### Reading and Processing a File Line by Line

```tardi
: process-file ( path -- )
    <reader>
    [ 
        read-line
        dup empty? !  // Continue while lines aren't empty
    ] [
        // Process each line here
        println  // Just print it for this example
    ] while
    close
;

"/tmp/data.txt" process-file
```

### Copying a File

```tardi
: copy-file ( source dest -- success )
    swap read-file  // Read source file
    [ write-file ] [ 2drop #f ] if  // Write to dest if read succeeded
;

"/tmp/source.txt" "/tmp/dest.txt" copy-file
```

### Logging with Timestamps

```tardi
: log-message ( message -- )
    "/tmp/app.log" <writer>
    swap over write-line
    close
;

"Application started" log-message
```

### Interactive Input Loop

```tardi
: input-loop
    <stdin>
    [ 
        "Enter text (empty to quit): " print
        read-line
        dup empty? !  // Continue while input isn't empty
    ] [
        "You entered: " print println
    ] while
    close
;

input-loop
```

## Performance Notes

- Writers buffer output by default; use `flush` or `close` to ensure data is written
- Readers are efficient for both line-by-line and bulk reading
- Console operations automatically flush output for immediate display
- File operations include proper error handling and resource cleanup