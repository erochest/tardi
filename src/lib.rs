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
use crate::vm::VM;

pub fn run_file(file_path: &std::path::Path, print_stack: bool) -> Result<()> {
    let script_text = std::fs::read_to_string(file_path)?;

    let tokens = parse(&script_text).into_iter().collect::<Result<Vec<_>>>()?;
    let chunk = compile(tokens);
    let mut vm = VM::new();
    vm.execute(chunk)?;

    if print_stack {
        vm.print_stack();
    }

    Ok(())
}
