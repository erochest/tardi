//! Tardi environmentming language implementation

pub mod compiler;
pub mod config;
pub mod core;
pub mod env;
pub mod error;
pub mod module;
pub mod scanner;
pub mod shared;
pub mod value;
pub mod vm;

use std::path::Path;

use error::VMError;
use rustyline::error::ReadlineError;
use rustyline::history::{FileHistory, History};
use rustyline::{self, DefaultEditor};

use crate::compiler::Compiler;
use crate::config::Config;
use crate::core::Tardi;
use crate::error::Result;
use crate::scanner::Scanner;
use crate::vm::VM;

/// Run a Tardi source file
pub fn run_file(path: &Path, _config: &Config, print_stack: bool) -> Result<()> {
    let mut tardi = Tardi::default();
    // TODO: add an option for the bootstrap dir
    tardi.bootstrap(None)?;
    tardi.execute_file(path)?;
    let tardi = tardi;

    if print_stack {
        // Print stack contents from top to bottom
        for value in tardi.stack() {
            eprintln!("{}", value.to_repr());
        }
    }

    Ok(())
}

// TODO: highlighting
// TODO: completion
// TODO: hints
// TODO: multilines (via rustyline::validate)
pub fn repl(config: &Config) -> Result<()> {
    let mut tardi = Tardi::from(config);

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
                readline.add_history_entry(&input)?;

                // TODO: reset the stack and items on it on errors
                // how? memory snapshots? clones? yech!
                match tardi.execute_str(&input) {
                    Ok(()) => println!("ok"),
                    Err(error::Error::VMError(VMError::Bye)) => {
                        println!("bye now");
                        break;
                    }
                    Err(err) => eprintln!("error: {}", err),
                }

                for value in tardi.stack() {
                    println!("{}", value.to_repr());
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
        readline.history_mut().save(history_file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests;
