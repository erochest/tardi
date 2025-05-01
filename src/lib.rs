//! Tardi environmentming language implementation

pub mod compiler;
pub mod config;
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
use rustyline::history::{FileHistory, History, MemHistory};
use rustyline::{self, DefaultEditor, EditMode, Editor};

// Re-exports
// TODO: clean these up
use crate::config::Config;
use crate::core::{Execute, Scan, Tardi};
pub use compiler::Compiler;
pub use env::Environment;
pub use error::Result;
pub use scanner::Scanner;
pub use value::Value;
pub use vm::VM;

/// Run a Tardi source file
pub fn run_file(path: &PathBuf, _config: Config, print_stack: bool) -> Result<()> {
    let source = std::fs::read_to_string(path)?;

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

// TODO: highlighting
// TODO: completion
// TODO: hints
// TODO: multilines (via rustyline::validate)
pub fn repl(config: Config) -> Result<()> {
    let mut tardi = Tardi::default();

    let rl_config = config.clone().into();
    let history = FileHistory::new();
    let mut readline = DefaultEditor::with_history(rl_config, history)?;
    if let Some(history_file) = config.repl.history_file.as_ref() {
        if history_file.exists() {
            readline.history_mut().load(history_file)?;
        }
    }

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

    if let Some(history_file) = config.repl.history_file.as_ref() {
        readline.history_mut().save(&history_file)?;
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
