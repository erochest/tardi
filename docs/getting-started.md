# Getting Started with Tardi

Welcome to Tardi, a stack-based concatenative programming language! This tutorial will introduce you to the fundamental concepts of stack-based programming and get you writing your first Tardi programs.

## Try It Yourself!

Want to follow along interactively? Start Tardi's REPL (Read-Eval-Print Loop) to try the examples as you read:

```bash
tardi --print-stack
```

This will give you a prompt where you can type Tardi commands and see the results immediately:

```
>>> 5 3 +
ok
8
```

Type any of the code examples from this tutorial to see them in action!

## What is a Stack-Based Language?

Unlike traditional languages where you write `add(5, 3)`, stack-based languages work with a data stack where you push values and then apply operations:

```tardi
5 3 +    // Push 5, push 3, then add them → 8
```

Think of the stack like a pile of plates - you can only add to the top (push) or remove from the top (pop). Operations take their arguments from the stack and put results back on the stack.

## Your First Tardi Commands

Let's start with some basic arithmetic:

```tardi
// Simple arithmetic
42          // Pushes 42 onto the stack
3 7 +       // Pushes 3, pushes 7, adds them → stack now has: 42 10
*           // Multiplies top two values → stack now has: 420
```

After running this, your stack contains `420`.

## Understanding Stack Effects

Every operation in Tardi has a "stack effect" that describes what it consumes and produces:

```tardi
// dup ( a -- a a ) - duplicates the top item
5 dup       // Stack: 5 5

// swap ( a b -- b a ) - swaps top two items
1 2 swap    // Stack: 2 1

// drop ( a -- ) - removes top item
1 2 3 drop  // Stack: 1 2
```

The notation `( before -- after )` shows what the operation does:
- Items before `--` are consumed from the stack
- Items after `--` are produced on the stack
- Rightmost items are at the top of the stack

## Why Stack-Based Programming?

Stack-based languages excel at **function composition**. Instead of nested function calls, you create pipelines:

```tardi
// Traditional: sqrt(abs(x - 5))
// Tardi: chain operations naturally
x 5 - abs sqrt

// Or with intermediate steps you can see:
10          // x = 10
5 -         // 10 - 5 = 5
dup *       // 5 * 5 = 25 (square instead of abs for this example)
```

Each step is clear and operations flow left to right.

## Concatenative Programming

Tardi is also **concatenative**, meaning you build programs by concatenating (joining) smaller programs. Any sequence of operations can become a function:

```tardi
// This sequence squares a number:
dup *

// We can turn it into a function:
: square ( n -- n^2 )
    dup *
;

// Now use it:
5 square    // Result: 25
```

The beauty is that `5 square` is exactly the same as writing `5 dup *` - functions are just named sequences of operations.

## Working with Different Data Types

Tardi supports various data types, all managed through the stack:

```tardi
// Numbers (integers and floats)
42          // Integer
3.14        // Float
2 3.5 +     // Mixed arithmetic → 5.5

// Booleans
#t          // True
#f          // False
5 3 >       // Comparison → #t

// Characters
'a'         // Single character
'Z'         // Another character

// Strings
"Hello, world!"     // Basic string
"Line 1\nLine 2"    // With escape sequences
"""
Multi-line
string here
"""
```

## Your First Tardi Program

Let's write a program that calculates the area of a circle:

```tardi
// Calculate circle area: π * r²
: circle-area ( radius -- area )
    dup *           // Square the radius (r²)
    3.14159 *       // Multiply by π
;

// Test it:
5 circle-area       // Area of circle with radius 5 → 78.53975
```

## Collections: Vectors

Vectors (arrays) are created with curly braces:

```tardi
// Create vectors
{ 1 2 3 4 5 }       // Vector of numbers
{ "a" "b" "c" }     // Vector of strings
{ }                 // Empty vector

// Vector operations
{ 1 2 3 } 4 over push!      // Add 4 to end → { 1 2 3 4 }
{ 1 2 3 4 } dup pop!        // Remove last → { 1 2 3 } 4
{ 1 2 3 } first             // Get first element → 1
```

## Control Flow with Conditionals

Tardi uses lambdas (anonymous functions) for control flow:

```tardi
// if statement: condition true-branch false-branch if
5 0 > [ "positive" ] [ "not positive" ] if

// when statement: condition action when
temperature 100 > [ "Water is boiling!" println ] when

// Working with the result:
: describe-number ( n -- description )
    dup 0 > [
        "positive"
    ] [
        dup 0 < [ "negative" ] [ "zero" ] if
    ] if
;

-5 describe-number  // Result: "negative"
```

## Stack Manipulation for Data Flow

Stack operations let you organize data for the operations you need:

```tardi
// Calculate both sum and product of two numbers
: sum-and-product ( a b -- sum product )
    2dup +          // 2dup copies both values, then add
    -rot *          // -rot moves sum below the original values, then multiply
;

3 7 sum-and-product // Result: 10 21 (sum=10, product=21)

// Convert temperature from Celsius to Fahrenheit
: celsius-to-fahrenheit ( c -- f )
    9 * 5 /         // Multiply by 9, divide by 5
    32 +            // Add 32
;

20 celsius-to-fahrenheit    // 20°C → 68°F
```

## Running Your First Script

### 1. Install Tardi

First, build Tardi from source. If you have [just](https://just.systems/) installed, it's extra easy.

```bash
git clone https://github.com/your-repo/tardi
cd tardi
just install
```

### 2. Create a Script File

Create a file called `hello.tardi`:

```tardi
// hello.tardi - My first Tardi program

// Define a greeting function
: greet ( name -- )
    "Hello, " swap concat
    "!" concat
    println
;

// Use it
"Tardi" greet
"World" greet

// Do some math
: calculate-tip ( bill-amount tip-percent -- tip total )
    dupd                // Copy bill
    *                   // Calculate tip amount
    dup rot +           // Calculate total (tip + bill)
;

50.00 18 calculate-tip  // 18% tip on $50 bill
"Tip: $" print . nl
"Total: $" print . nl
```

### 3. Run the Script

```bash
tardi hello.tardi
```

Or run it with stack printing to see what's left on the stack:

```bash
tardi --print-stack hello.tardi
```

### 4. Interactive REPL

Start an interactive session to experiment:

```bash
tardi         # Starts the REPL (Read-Eval-Print Loop)
# Or explicitly:
tardi repl    # Same as above
```

In the REPL, you can try commands immediately:

```
>>> 5 3 +
ok
8
>>> "Hello" " " "World" concat concat
ok
"Hello World"
>>> { 1 2 3 } [ dup * ] map
ok
{ 1 4 9 }
```

## Creating Your First Module

Modules let you organize and share code. Let's create a math utilities module.

### 1. Create the Module File

Create `math-utils.tardi`:

```tardi
// math-utils.tardi - Mathematical utility functions

// Export the functions we want other modules to use
exports: square cube abs factorial even? odd? ;

// Square a number
: square ( n -- n^2 )
    dup *
;

// Cube a number
: cube ( n -- n^3 )
    dup square *
;

// Absolute value
: abs ( n -- |n| )
    dup 0 < [ -1 * ] when
;

// Factorial (recursive)
: factorial ( n -- n! )
    dup 1 <= [
        drop 1
    ] [
        dup 1 - factorial *
    ] if
;

// Check if number is even
: even? ( n -- boolean )
    2 % 0 ==
;

// Check if number is odd
: odd? ( n -- boolean )
    even? !
;
```

### 2. Use the Module

Create `main.tardi` that uses your module:

```tardi
// main.tardi - Using our math utilities

// Import our custom module
uses: math-utils

// Import standard library modules
uses: std/io

// Test our functions
: test-math-functions
    "Testing math functions:" println

    5 square "5² = " swap >string concat println
    3 cube "3³ = " swap >string concat println
    -7 abs "|-7| = " swap >string concat println
    5 factorial "5! = " swap >string concat println

    4 even? [ "4 is even" ] [ "4 is odd" ] if println
    7 odd? [ "7 is odd" ] [ "7 is even" ] if println
;

test-math-functions
```

### 3. Run Your Program

```bash
tardi main.tardi
```

Output:
```
Testing math functions:
5² = 25
3³ = 27
|-7| = 7
5! = 120
4 is even
7 is odd
```

## Working with Standard Library Modules

Tardi includes several standard library modules:

### I/O Operations

```tardi
uses: std/io

// Write to a file
"Hello, file!" "/tmp/greeting.txt" write-file

// Read from a file
"/tmp/greeting.txt" read-file println drop

// Console output
"What's your name? " print
<stdin> read-line "Hello, " swap concat "!" concat println
close
```

### Hash Maps (Dictionaries)

```tardi
uses: std/hashmaps

// Create a hash map
H{ { "name" "Alice" } { "age" 30 } { "city" "New York" } }

// Access values
"name" over get println drop    // Prints: Alice

// Add/update values
"country" "USA" rot set!
"age" over get drop 1 + "age" rot set!
```

### Vectors with Higher-Order Functions

```tardi
uses: std/vectors

// Transform data
{ 1 2 3 4 5 } [ dup * ] map         // Square each: { 1 4 9 16 25 }
{ 1 2 3 4 5 } [ 2 % 0 == ] filter   // Keep evens: { 2 4 }
{ 1 2 3 4 5 } 0 [ + ] reduce        // Sum all: 15

// Process each element
{ "apple" "banana" "cherry" } [
    "I like " swap concat println
] each
```

## Next Steps

Now that you understand the basics, explore these topics:

1. **Advanced Stack Manipulation** - Learn combinators like `keep`, `dip`, and `2dup`
2. **Function Composition** - Build complex operations from simple functions
3. **Error Handling** - Use conditional logic and success flags
4. **File Processing** - Read, transform, and write data files
5. **Module Development** - Create reusable libraries

### Key Concepts to Remember

- **Everything goes through the stack** - master stack manipulation
- **Function composition** - build complex operations step by step
- **Concatenative style** - any sequence of operations can become a function
- **Module system** - organize code with `uses:` and `exports:`
- **Interactive development** - use the REPL to experiment

### Practice Exercises

Try these challenges to solidify your understanding:

1. Write a function that converts Fahrenheit to Celsius
2. Create a function that finds the largest number in a vector
3. Build a word counter that takes a string and returns a hash map of word frequencies
4. Write a module for geometric calculations (areas, perimeters, volumes)
5. Create a simple calculator that processes a series of operations

Welcome to the world of stack-based programming with Tardi! The more you practice thinking in terms of data flow and function composition, the more natural it becomes.
