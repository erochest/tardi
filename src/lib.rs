pub mod chunk;
pub mod compiler;
pub mod error;
pub mod op_code;
pub mod parser;
pub mod value;
pub mod vm;

use crate::compiler::compile;
use crate::error::Result;
use crate::parser::parse;
use std::io::Write;

use crate::vm::VM;

pub fn run_file(file_path: &std::path::Path, print_stack: bool) -> Result<()> {
    let script_text = std::fs::read_to_string(file_path)?;

    let tokens = parse(&script_text)?;
    let chunk = compile(tokens);
    let mut vm = VM::new();
    vm.execute(chunk)?;

    if print_stack {
        vm.print_stack();
    }

    Ok(())
}

pub fn run_repl(print_stack: bool) -> Result<()> {
    // TODO: print banner
    let mut vm = VM::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;

        if line.trim() == "/quit" {
            break;
        }

        let tokens = parse(&line)?;
        // TODO: this may not have any memory between lines of input
        // of functions defined, etc. fix this.
        let chunk = compile(tokens);
        vm.execute(chunk)?;

        if print_stack {
            vm.print_stack();
        }
    }

    Ok(())
}
