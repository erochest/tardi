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