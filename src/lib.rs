//! Tardi environmentming language implementation

pub mod compiler;
pub mod core;
pub mod env;
pub mod error;
pub mod scanner;
pub mod shared;
pub mod value;
pub mod vm;

use std::io::{self, Write};
use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{self, Config, DefaultEditor, EditMode, Editor};

// Re-exports
// TODO: clean these up
use crate::core::{Execute, Scan, Tardi};
pub use compiler::Compiler;
pub use env::Environment;
pub use error::Result;
pub use scanner::Scanner;
pub use value::Value;
pub use vm::VM;

/// Run a Tardi source file
pub fn run_file(path: &PathBuf, print_stack: bool) -> Result<()> {
    let source = std::fs::read_to_string(path)?;

    // when scanner gets MACRO: what needs to happen?
    // - it needs to read the MACRO definition immediately
    // - it hands those to the compiler
    // - it adds a function to the Environment under that name, with an `immediate` flag set
    // when it then reads the macro token
    // - it takes the tokens so far and passes the token, them, and itself to `VM::run_macro`
    // - `run_macro' places the token vector on the stack runs the macro word
    // - it allows the function to modify the environment and the token vector on the stack

    let mut tardi = Tardi::default();
    // TODO: add an option for the bootstrap dir
    tardi.bootstrap(None)?;
    tardi.execute_str(&source)?;
    let tardi = tardi;

    if print_stack {
        // Print stack contents from top to bottom
        for value in tardi.stack() {
            eprintln!("{}", value);
        }
    }

    Ok(())
}

// TODO: configuration
// TODO: configure emacs or vi on configuration or command-line
// TODO: history
// TODO: highlighting
// TODO: completion
// TODO: hints
// TODO: multilines (via rustyline::validate)
pub fn repl() -> Result<()> {
    let mut tardi = Tardi::default();

    let rl_config = Config::builder().edit_mode(EditMode::Emacs).build();
    let mut readline = DefaultEditor::with_config(rl_config)?;

    tardi.bootstrap(None)?;

    loop {
        let input = readline.readline(">>> ");
        match input {
            Ok(input) => {
                if is_quit(&input) {
                    println!("bye");
                    break;
                }

                readline.add_history_entry(&input)?;
                // TODO: reset the stack and items on it on errors
                // how? memory snapshots? clones? yech!
                match tardi.execute_str(&input) {
                    Ok(()) => println!("ok"),
                    Err(err) => eprintln!("error: {}", err),
                }

                for value in tardi.stack() {
                    println!("{}", value);
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("bye");
                break;
            }
            Err(err) => {
                eprintln!("ERROR: {}", err);
            }
        }
    }

    Ok(())
}

/// Returns true if the command indicates the user wants to stop.
fn is_quit(input: &str) -> bool {
    let input = input.trim();
    input == "/quit" || input == "/exit" || input == "/q" || input == "/xt"
}

#[cfg(test)]
mod tests;
