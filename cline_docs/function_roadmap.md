In active_context.md, the next thing to work on is variables, but concatenative languages often don't have variables as first-class entities. Instead, they use the global data stack for that, and then they implement variables using metaprogramming. It does make sense to start working toward functions, but we have a lot of steps to go to see that through. Here's what I'm thinking we need to do. Each header just gives an overview. We'll go into more detail on each feature as we get to it.

## Shared Values

Since some of the work coming up will involve complex objects that will get shared and could get pushed onto the stack multiple times, part of this will be moving to using `Rc<RefCell<Value>>` almost everywhere we're using `Value` now. We can define some things in Rust to make this easier:

- `type SharedValue = Rc<RefCell<Value>>`
- `fn shared(value: Value) -> SharedValue`

## Return Stack

This will be a separate stack. It will primarily be used for function calls, but it will be available for temporary storage by the user.

Here are the words that the user can use to interact with the return stack:

- `>r` or _to R_. This pops the top of the data stack and pushes it onto the R stack
- `r>` or _from R_. This pops the top of the R stack and pushes it onto the R stack
- `r@` or _copy R_. This copies the top of the R stack onto the data stack

Note: The W register implementation has been postponed until a specific use case arises.

## Character Values and Literals

Characters are pretty low-level and a relatively light lift, but they'll be used more in strings.

A character is quoted with single quotes. These are some examples:

```
'a' 'A' '4' ' ' '\t' '\n' '\'' '"'
```

There is also some syntax for unicode characters:

```
'\u61' '\u7a' '\u7A' '\u{2f}' '\u{1f642}'
```

The full list of escape characters are:

- '\n'
- '\r'
- '\t'
- '\''
- '\"'
- '\uff' (for ASCII)
- '\u{ff}' (for all Unicode characters)

These will be represented by a `Value::Char(char)`

## List Objects

List literals will come later, but this will enable us to create list objects and perform some simple, basic operations on them.

Here are the words I'd like to start implementing:

- `<list>` ( -- list ) creates a list value and pushes it onto the stack
- `append` ( x list  -- ) adds a value onto the end of a list
- `prepend` ( x list -- ) adds on value onto the beginning of a list
- `concat` ( list1 list2 -- list3 ) concatenates two lists
- `split-head!` ( list -- a ) removes the head from a list and pushes it onto a stack

Lists will be stored in `Value::List(Vec<Value>)`

Note: The initial implementation will focus on these basic operations. Future enhancements, such as iterators and additional protocols, will be considered in later stages of development.

## String Objects and Literals

Strings will be the first complex object that we add. We'll implement basic string operations to start with, focusing on creation, conversion, and simple manipulations.

String values will be stored in `Value::String(String)`

Note: The initial implementation will keep string operations minimal. More advanced string manipulation functions may be added in future iterations as needed.

### String Literals

String literals will be marked with double quotes. It supports the same escape sequences as characters do.

```
"a string"
"another string"
"a string with \"quotes\" in it"
"a string\nwith escaped\n\tcharacters."
"this is an expressionless face: \u{1f611}"
```

There is also a long string literal that is delimited by triple double-quotes:

```
"""
This is one long string.
With embedded newlines.
It spans multiple lines.
\tIt can also contain escaped characters.
"But it doesn't have to."
"""
```

## Function and Lambda Objects

Now we can start with function objects. These will be `Value` instances that contain a `Callable` enum with these fields.

The enum has two members. One is `Callable::BuiltIn(OpFn)`. The other is `Callable::Fn(Function)`. The `Function` struct, has these fields:

- `name` -- a string or word. If this is optional, we can use these structures for lambdas as well
- `words` -- the list of words that this was compiled from. this is mainly for printing later.
- `instructions` -- this is a pointer to the beginning of the instructions in the main VM instructions

I think we'll need these words to start:

- `<function>` ( name word-list -- function ) compiles and constructs the function and places it on the stack. This also installs it in the VM and compiler to be used by other code
- `<lambda>` ( word-list -- lambda ) creates a lambda object, which is just a `Function` with the word omitted

### Compilation

Because lambdas may nest inside functions or even other lambdas, we don't want to just compile these straight. We'd need to add too many jumps.

To get around this, we should have the compiler maintain a stack of functions/lambdas that it's compiling. Each item in the stack is a vector of instructions. When that function/lambda is finished, its instructions are popped from the stack and appended to the Program with the `extend_instructions` method, which returns the position of the first instruction in the new just added. When the program adds this, it first adds a jump over the function. So if you add the lambda op codes `lit 4 add return` when the program has 12 instructions, then the program would look like this from that point on:

| pos | op     |
+-----+--------+
|  13 | jump   |
|  14 | 19     |
|  15 | lit    |
|  16 | 4      |
|  17 | add    |
|  18 | return |

And `extend_instructions` would return `15`, since that's the beginning of the codes added.

#### Compiling Functions

Compiling functions happen in the `<function>` word. It follows these steps:

1. Add a default `Function` in the `Program.op_table` and allocate one under its name in the `Program.op_map`. This should allow us to handle recursion.
2. Create a new instruction vector on the compilation stack to add the instructions to.
3. Compile the instructions in the body of the function.
4. When finished, pop the instructions off the compilation stack and add it to the main instruction vector with `Program::extend_instructions`
5. With the address for the function code, set the address for the function in the `op_table`.

#### Compiling Lambdas

1. Create a new instruction vector on the compilation stack to add the instructions to.
2. Compile the instructions in the body of the function.
3. When finished, pop the instructions off the compilation stack and add it to the main instruction vector with `Program::extend_instructions`
4. Add the `Function` to the constants table and emit a `lit` opcode to retrieve it.

### New Op Codes

- `Call` -- This gets its argument, which is an index in the `op_table`. It retrieves that object and calls it either by calling the `OpFn` or jumping to its `Address`
- `CallStack` -- This gets a lambda from the top of the stack and jumps to its `Address`
- `Ip` -- This takes the current ip and pushes it to the top of the stack. Then `1 +` and `>r` can move the next position to the return stack
- `Jump` -- This takes the next argument and jumps the IP to there.
- `JumpStack` -- This takes the `Address` on top of the stack and jumps the IP to there.
- `Return` -- This pops the top of the return stack and moves the IP to there

### Executing Functions

When the compiler compiles a word, it looks it up in the `op_map`. From there it gets its index in `op_table`, and it adds that as an argument to `Call`.

## Comments

This is mainly so we can add documentation to the initialization script coming up. Comments should start with `//` and go until the end of the line.

## Initialization Script

At this point, we can start building the system within itself, and having an initialization script can help us do this. Here are my thoughts:

- This would be the script `src\init.tardi`
- It gets embedded in the compiled binary
- There's a `--init-script` command-line argument that uses an alternative script

After the system is initialized, that script is scanned, compiled, and executed before processing the other scripts that are passed in on the command line.

The definition for list literals above would be a good start for the initialization script.

We can define `over` at this point using this in the initialization script:

```
"over" { >r dup r> swap } <function>
```

## Compiler Words

This takes a lambda/function object and compiles it.

- `compile` -- ( lambda -- )

### Open Questions

- what does `compile` do?
- is the compiled code in the main instruction list and the lambda just has an index/pointer to its location or does it contain the instructions itself?

## Scanner/Parser Words

To facilitate metaprogramming, the user will need access to some of the functionality of the `Scanner`. It can be through these words:

- `scan-word` ( -- token ) this reads one token and leaves it on top of the stack.
- `scan-string` ( end -- string ) this reads the input as a string until it gets to `end`, and it leaves this string on the top of the stack.
- `scan-tokens` ( end -- token-list ) this reads the input as raw token words and leaves these in a list on top of the stack.
- `scan-values` ( end -- value-list ) this reads the input as values and leaves these in a list on top of the stack. This makes use of the metaprogramming framework, which means that it can trigger reading more macros. This would allow it to parse nested structures.

Unless these are called during scanning by using metaprogramming, these will throw errors.

## Metaprogramming

Metaprogramming is defined by a `MACRO:` definition, which looks like this:

```
MACRO: TRIGGER-WORD WORD DEFINITIONS ;
```

Macros are called with the current list of tokens for that file on the stack. The macro may make changes to that list, but is expected to leave it on the stack.

So for instance, we can define list literals (along with a utility word) like this:

```
MACRO: { } scan-values over append ;
```

In this case, it leverages the standard behavior of `scan-values`, which creates a list, and it leave that on the stack. The macro appends that list to the list of values being read in the file.

## Lambdas Literals

With metaprogramming and the initialization script, we can introduce lambda literals. These will be curly-brace delimited lists of code and operations. They get read into a lambda object and compiled.

```
MACRO: { } scan-values >lambda dup compile over append ;
```

## Function Literals

This would be a macro in the initialization file:

```
MACRO: : scan-word ; scan-values >lambda <function> ;
```

`over` would be a great word to use to test and start on this:

```
// ( x y -- x y x )
: over >r dup r> swap ;
```

### Future Function Extensions

This will get us started, but we can add more to functions at some point:

- stack effect statements like `: over ( x y -- x y x ) ... ;`
- documentation comment before the function declaration using triple comments (`///`)

## Implementation Order and Considerations

1. The features will be implemented in the order presented in this roadmap, starting with Shared Values.
2. We will monitor the performance impact of using `Rc<RefCell<Value>>` and address any issues as they arise.
3. We'll need to be cautious about potential reference cycles when working with shared values.
4. The existing testing strategy (input/output tests + unit tests) will be maintained throughout the implementation.
5. Documentation will be updated for each major feature as it's implemented.
6. Future enhancements, such as traits/protocols and more advanced iterators, are planned for later stages of development.

Remember that this roadmap is flexible, and adjustments may be made as we progress through the implementation.
