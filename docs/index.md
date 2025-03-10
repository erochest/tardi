# Tardi

## Concatenative Languages

### Stacks

#### Data Stack

#### Call Stack

## Types

- integer (64-bit)
- float (64-bit)
- rational
- boolean
- string
- vector
- function
- lambda
- address

## Comments

Comments start with the hash and go to the end of the line.

```
# This is a comment.
42    # <- This answer.
```

## Words and Operations

### Stack Manipulation

`drop`

`dup`

`nip`

`over`

`pop`

`rot`

`swap`

### Flow Control

`call`

`if`

`return`

### Operators

`+`

`-`

`*`

`/`

`%`

`==`

`!=`

`<`

`>`

`<=`

`>=`

`!`

### Call Stack Operators

`>r`

`r>`

`r@`

`IP`

### Conditionals

#### `if`

**Stack effect**: `( ? then-clause: ( ...s -- ...s' ) else-clause: ( ...s -- ...s' ) )

For example, this poorly factored word determines if a number is odd or even and prints out the result:

```
: print-even ( n -- )
  2 % 0 ==
  [ "even" ]  [ "odd" ] if
  println ;
```

### Op Code Words

`-call-tardi-fn-`

`-get-constant-`

`-jump-`

`-mark-jump-`

## Function Definition

Functions are defined using this format:

```
## Documentation String
: NAME ( STACK -- EFFECT ) WORDS ... ;
```

`:` and `;` are literals.

`NAME` is the word that your function will be called by.

`( STACK -- EFFECT )` is the stack effect of this word. Left of the dashes are the input stack, and to the right are the stack outputs.

`WORDS ...` are the words that define the implementation of this function.

### Documentation Strings

Any comments immediately preceding the function that begin with two hashes are documentation comments. They're stored with the function.

Maybe someday we'll do something with them.

### Stack Effects

### Recursion

## Metaprogramming

Metaprogramming in Tardi is achieved using compile-time words. This is triggered by the `MACRO:` word. The syntax for this word is:

```
MACRO: TOKEN definition... ;
```

When `TOKEN` is read as Tardi is scanning the source code, the rest of the definition is triggered. This has the core Tardi words available to them. The stack will be empty, but there are several words available to scan ahead through the stack. These read forward to the next occurance of a token and return the source code scanned in different formats. Anything on the stack after the macro executes is appended to the values being read by the scanner.

`lookahead-string ( TOKEN -- STRING )` This reads forward until it reads `TOKEN` and returns the characters read as a single string.

`lookahead-tokens ( TOKEN -- STRING-LIST )` This reads forward until it reads `TOKEN` and it returns the words read as a list of strings.

`lookahead-values ( TOKEN -- VALUE-LIST )` This reads forward until it reads `TOKEN` and it parses and converts these strings into values. This may involve invoking other macros.

- [ ] TODO: Does this need to have a vector of previously scanned words available on the stack?
- [ ] TODO: define "core words."

### Example

The most simple example is for defining lists. Essentially, this is defined in this way:

```
MACRO: [ ] lookahead-values ;
```

That is, it parses the input ahead to the end of the list, and it leaves that list on the stack.
