# tardi

Tardi is a stack-based language. It's a project for me to play with implementing a simple interpreted language. Maybe it will also be compiled someday.

## Status

Currently it is _very_ rough.

It's probably broken in many disappointing ways. There will definitely be breaking changes in its future.

What's still to do?

- **Garbage collection**. Right now it uses reference counting, and creating a memory leak is trivial by creating circular structures.
- **Types**. One of the things I really want to play with is types. I haven't started that yet. When I do. Things _will_ break.
- **Error messages**. Yep. When things don't work, you're pretty much in the dark. It sucks.
- **Error handling**. Currently it just fails with an unhelpful message. It needs the ability to catch errors and deal with them in the program.
- **Local variables**. Stack-based languages don't require them or lean on them the way other languages do, but they're still nice. I'd like to add them.
- **User-defined data types**. I need to add support for traits/protocols/what-have-yous, structures, enums, and other goodies. I'm waiting for type checking and type inference.

For a full list, run this from the command line:

```bash
rg --ignore-case "\\bxxx\\b|\\btodo\\b" docs src tests
```

Or check out the [todo.md](todo.md) file for an unordered list of things I'd like to get to.

## Getting Started

If I haven't scared you off already, there's [a short quick-start tutorial](/docs/getting-started.md). I've left myself some notes about what's currently aspirational in there, and it's honestly not as much as I was thinking it might be.

### Installing

To build and install it, you'll need [rust](https://rustup.rs/) installed. I'm using [just](https://just.systems/) for automating simple tasks, like installation, so while not required, it will make things simpler.

With `just`, you can just do:

```bash
just install
```

Without `just`, it's still as easy as 1-2-3. Kind of.

```bash
cargo install --path .
-mkdir -p "$TARDI_DATA_DIR"
-cp -r std/* "$TARDI_DATA_DIR"
```

Make sure that the cargo installation location is on your path. By default it's `~/.cargo/bin`.

Now you'll need to figure out what your platform's `$TARDI_DATA_DIR` is, relative to the  standard user data directory. Values of this for common platforms are:

- Linux: `~/.local/share/tardi`
- Windows: `%USERPROFILE%\AppData\Roaming\Tardi\data`
- MacOS: `~/Library/Application Support/Tardi`

Copy the `./std` directory and its contents into the location given above. If this is your first time installing Tardi, you'll probably need to create that directory first.

### Running the REPL

You can run Tardi interactively by just running the command. The `--print-stack` command option in helpful: it causes Tardi to print the stack after each command:

```bash
tardi --print-stack
```

### Running a Script

If you have a script or module(s) and script, you can run it by passing it to the program:

```bash
tardi hello-world.tardi
```
