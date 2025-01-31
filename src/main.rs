use clap::Parser;
use clap_verbosity_flag::Verbosity;
use env_logger;
use human_panic::setup_panic;
use std::convert::TryFrom;

use tardi::error::{Error, Result};
use tardi::parser::parse;
use tardi::chunk::Chunk;
use tardi::value::Value;
use tardi::op_code::OpCode;
use tardi::compiler::compile;


fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let script_path = &args.script_file;
    let script_text = std::fs::read_to_string(script_path)?;

    let tokens = parse(&script_text).into_iter().collect::<Result<Vec<_>>>()?;
    let chunk = compile(tokens);
    let mut vm = VM::new();
    vm.execute(chunk)?;

    if args.print_stack {
        vm.print_stack();
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    /// The Tardi script file to execute
    script_file: std::path::PathBuf,

    /// Print the stack after execution
    #[arg(long)]
    print_stack: bool,
}

struct VM {
    stack: Vec<Value>,
}

impl VM {
    fn new() -> Self {
        VM {
            stack: Vec::new(),
        }
    }

    fn execute(&mut self, chunk: Chunk) -> Result<()> {
        let mut ip = 0;
        
        while ip < chunk.code.len() {
            let instruction = chunk.code[ip];
            
            match OpCode::try_from(instruction)? {
                OpCode::GetConstant => {
                    ip += 1;
                    let constant_idx = chunk.code[ip];
                    let constant = chunk.constants[constant_idx as usize].clone();
                    self.stack.push(constant);
                },
                OpCode::Add => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push((a + b)?);
                },
                OpCode::Sub => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push((a - b)?);
                },
                OpCode::Mult => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push((a * b)?);
                },
            }
            
            ip += 1;
        }

        Ok(())
    }

    fn print_stack(&self) {
        for value in &self.stack {
            eprintln!("{}", value);
        }
    }
}