pub mod builtins;
pub mod chunk;
pub mod compiler;
pub mod error;
pub mod op_code;
pub mod scanner;
pub mod value;
pub mod vm;

use crate::compiler::compile;
use crate::error::Result;
use std::io::Write;

use crate::vm::VM;

pub fn run_file(file_path: &std::path::Path, print_stack: bool) -> Result<()> {
    let input_text = std::fs::read_to_string(file_path)?;
    let mut chunk = compile(&input_text)?;
    log::trace!("chunk.constants = {:?}", chunk.constants);
    log::trace!("chunk.code      = {:?}", chunk.code);
    let mut vm = VM::new();
    vm.execute(&mut chunk)?;

    if print_stack {
        vm.print_stack();
    }

    Ok(())
}

pub fn run_repl(print_stack: bool) -> Result<()> {
    // TODO: print banner
    // TODO: catch errors
    let mut vm = VM::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let line = line.trim();

        if line == "/quit" || line == "/exit" {
            break;
        }

        // TODO: this may not have any memory between lines of input
        // of functions defined, etc. fix this.
        let mut chunk = compile(&line)?;
        vm.execute(&mut chunk)?;

        if print_stack {
            vm.print_stack();
        }
    }

    Ok(())
}
