use clap::Parser;
use clap_verbosity_flag::Verbosity;
use env_logger;
use human_panic::setup_panic;
use std::ops::Add;
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

enum TokenType {
    Integer(i64),
    Plus,
}

struct Token {
    token_type: TokenType,
    line_no: usize,
    column: usize,
    length: usize,
}

impl TryFrom<&str> for TokenType {
    type Error = Error;

    fn try_from(word: &str) -> result::Result<Self, Self::Error> {
        if let Ok(number) = word.parse::<i64>() {
            Ok(TokenType::Integer(number))
        } else if word == "+" {
            Ok(TokenType::Plus)
        } else {
            Err(Error::InvalidToken(word.to_string()))
        }
    }
}

fn parse(input: &str) -> Vec<Result<Token>> {
    input.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let token_type = TokenType::try_from(word)
                .map_err(|e| Error::InvalidToken(word.to_string()))?;
            
            Ok(Token {
                token_type,
                line_no: 1,
                column: i,
                length: word.len(),
            })
        })
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

#[derive(Clone, Debug)]
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

impl Add for Value {
    type Output = result::Result<Value, Error>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            // (a, b) => Err(Error::InvalidOperands(a.to_string(), b.to_string())),
        }
    }
}

#[repr(u8)]
enum OpCode {
    GetConstant = 0,
    Add,
}

impl TryFrom<u8> for OpCode {
    type Error = Error;

    fn try_from(value: u8) -> result::Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::GetConstant),
            1 => Ok(OpCode::Add),
            code => Err(Error::InvalidOpCode(code)),
        }
    }
}

fn compile(tokens: Vec<Token>) -> Chunk {
    let mut chunk = Chunk::new();
    let mut current = 0;
    
    while current < tokens.len() {
        let token = &tokens[current];
        match token.token_type {
            TokenType::Integer(number) => {
                let constant = chunk.add_constant(Value::Integer(number));
                chunk.code.push(OpCode::GetConstant as u8);
                chunk.code.push(constant as u8);
            },
            TokenType::Plus => {
                chunk.code.push(OpCode::Add as u8);
            },
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
                OpCode::Add => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push((a + b)?);
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