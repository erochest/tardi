# String Operations

This document describes the string operations available in the Tardi language.

## String Literals

Strings can be created using double quotes:

```
"Hello, world!"
```

For multi-line strings, use triple double quotes:

```
"""
This is a multi-line
string that preserves
line breaks.
"""
```

### Escape Sequences

The following escape sequences are supported in string literals:

- `\n` - Newline
- `\r` - Carriage return
- `\t` - Tab
- `\"` - Double quote
- `\'` - Single quote
- `\\` - Backslash
- `\uXX` - ASCII character (2 hex digits)
- `\u{XXXX}` - Unicode character (1-6 hex digits)

Examples:
```
"Line 1\nLine 2"              // Two lines
"Tab\tindented"               // Contains a tab
"Unicode: \u{1F642}"         // Contains a smiling face emoji
"ASCII: \u41\u42\u43"        // "ABC" using ASCII codes
```

## String Operations

### Creating Empty Strings

The `<string>` word creates a new empty string:

```
<string>  // Creates an empty string
```

### Converting Values to Strings

The `>string` word converts any value to its string representation:

```
42 >string        // "42"
3.14 >string     // "3.14"
#t >string       // "#t"
'A' >string      // "'A'"
{ 1 2 } >string  // "[1 2]"
```

### UTF-8 Conversion

The `utf8>string` word converts a list of UTF-8 byte values to a string:

```
<list>
dup 72 swap append   // H
dup 101 swap append  // e
dup 108 swap append  // l
dup 108 swap append  // l
dup 111 swap append  // o
utf8>string         // "Hello"
```

### String Concatenation

The `string-concat` word concatenates two strings:

```
"Hello, " "world!" string-concat  // "Hello, world!"
"" "test" string-concat          // "test"
"prefix" "" string-concat        // "prefix"
```

Multiple strings can be concatenated by chaining operations:

```
"a" "b" string-concat "c" string-concat "d" string-concat  // "abcd"
```

## Implementation Details

### Shared Value System
Strings are implemented as `Value::String(String)` and are managed through the shared value system using `Rc<RefCell<Value>>`. This enables:
- Efficient string sharing between different parts of the program
- Mutable access when needed (e.g., for concatenation)
- Proper memory management through reference counting

### Memory Management
- Strings are heap-allocated and reference-counted
- The shared value system ensures proper cleanup when strings are no longer needed
- String operations create new strings rather than modifying existing ones

## Stack Effects

```
<string>      ( -- string )
>string       ( value -- string )
utf8>string   ( list -- string )
string-concat ( string1 string2 -- string3 )
```

## Error Handling

The following operations will result in errors:
- Attempting to concatenate non-string values
- Converting invalid UTF-8 byte sequences to strings
- Using malformed escape sequences in string literals
- Unterminated string literals

## Examples

### Basic String Manipulation
```
// Creating and concatenating strings
"Hello" " " string-concat "World" string-concat  // "Hello World"

// Converting numbers to strings
42 >string "=" string-concat 42 >string string-concat  // "42=42"

// Using escape sequences
"Line1\nLine2\tTabbed"  // Two lines, second line tabbed
```

### Function Examples
```
// Function to wrap text in parentheses
: parenthesize ( str -- str )
    "(" string-concat ")" string-concat
;

// Function to repeat a string
: repeat2 ( str -- str )
    dup string-concat
;
```

### Working with Unicode
```
// Creating strings with Unicode characters
"Unicode: \u{1F600}"  // Grinning face emoji
"Mixed: A\u{1F642}B"  // ASCII and Unicode mixed

// Converting Unicode code points
{ 0x1F600 } utf8>string  // Grinning face emoji
