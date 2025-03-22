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
