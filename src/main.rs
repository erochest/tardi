use clap::Parser;
use clap_verbosity_flag::Verbosity;
use env_logger;
use human_panic::setup_panic;
use std::{fmt, result};
use std::convert::TryFrom;

use tardi::error::{Error, Result};

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let script_path = &args.script_file;
    let script_text = std::fs::read_to_string(script_path)?;

    let tokens = parse(&script_text);
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

fn parse(input: &str) -> Vec<String> {
    input.split_whitespace()
        .map(String::from)
        .collect()
}

struct Chunk {
    constants: Vec<Value>,
    code: Vec<u8>,
}

impl Chunk {
    fn new() -> Self {
        Self { code: Vec::new(), constants: Vec::new() }
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}

#[derive(Clone)]
enum Value {
    Integer(i64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
        }
    }
}

#[repr(u8)]
enum OpCode {
    GetConstant = 0,
}

impl TryFrom<u8> for OpCode {
    type Error = Error;

    fn try_from(value: u8) -> result::Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::GetConstant),
            code => Err(Error::InvalidOpCode(code)),
        }
    }
}

fn compile(tokens: Vec<String>) -> Chunk {
    let mut chunk = Chunk::new();
    let mut current = 0;
    
    while current < tokens.len() {
        if let Ok(number) = tokens[current].parse::<i64>() {
            let constant = chunk.add_constant(Value::Integer(number));
            chunk.code.push(OpCode::GetConstant as u8);
            chunk.code.push(constant as u8);
        } else {
            todo!("compilation: {:?}", tokens[current]);
        }
        current += 1;
    }
    
    chunk
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