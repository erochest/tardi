//! Tardi environmentming language implementation

pub mod compiler;
pub mod env;
pub mod error;
pub mod scanner;
pub mod shared;
pub mod value;
pub mod vm;

use std::io::{self, Write};
use std::path::PathBuf;

// Re-exports
use crate::shared::{shared, Shared};
pub use compiler::Compiler;
pub use env::Environment;
pub use error::Result;
pub use scanner::Scanner;
use scanner::{Token, TokenType};
use value::Function;
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

pub fn repl() -> Result<()> {
    let mut tardi = Tardi::default();
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        let mut input = String::new();
        stdout.write_all(b"> ")?;
        stdout.flush()?;
        stdin.read_line(&mut input)?;
        let command = input.trim();
        if command == "/quit" || command == "/exit" || command == "/q" || command == "/xt" {
            println!("ok");
            break;
        }

        tardi.execute_str(&input)?;
        for value in tardi.stack() {
            println!("{}", value);
        }
    }

    Ok(())
}

pub trait Scan {
    fn scan(&mut self, input: &str) -> Result<Vec<Result<Token>>>;
    fn set_source(&mut self, input: &str);
    fn scan_token(&mut self) -> Option<Result<Token>>;
    fn scan_tokens_until(&mut self, token_type: TokenType) -> Result<Vec<Result<Token>>>;
    fn read_string_until(&mut self, delimiter: &str) -> Result<String>;
}

pub trait Compile {
    fn compile<S: Scan, E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut S,
        input: &str,
    ) -> Result<()>;
    fn compile_lambda<S: Scan, E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut S,
        input: &str,
    ) -> Result<()>;
}

pub trait Execute {
    fn run(&mut self, env: Shared<Environment>) -> Result<()>;
    fn stack(&self) -> Vec<Value>;
    fn execute_macro(
        &mut self,
        env: Shared<Environment>,
        trigger: &TokenType,
        function: &Function,
        tokens: &[Value],
    ) -> Result<Vec<Value>>;
}

pub struct Tardi {
    input: Option<String>,
    environment: Shared<Environment>,
    scanner: Scanner,
    compiler: Compiler,
    executor: VM,
}

impl Tardi {
    pub fn new(
        environment: Environment,
        scanner: Scanner,
        compiler: Compiler,
        executor: VM,
    ) -> Self {
        Tardi {
            input: None,
            environment: shared(environment),
            scanner,
            compiler,
            executor,
        }
    }

    pub fn reset(&mut self) {
        self.input = None;
    }

    pub fn scan_str(&mut self, input: &str) -> Result<Vec<Result<Token>>> {
        log::debug!("input : {:?}", input);
        let input = input.to_string();
        self.input = Some(input);
        Scan::scan(&mut self.scanner, self.input.as_ref().unwrap())
    }

    pub fn compile(&mut self, input: &str) -> Result<Shared<Environment>> {
        self.compiler.compile(
            &mut self.executor,
            self.environment.clone(),
            &mut self.scanner,
            input,
        )?;
        Ok(self.environment.clone())
    }

    pub fn execute(&mut self) -> Result<()> {
        log::debug!("environment:\n{:?}", self.environment.borrow());
        self.executor.run(self.environment.clone())
    }

    pub fn execute_str(&mut self, input: &str) -> Result<()> {
        self.reset();
        self.compile(input)?;
        self.execute()
    }

    pub fn stack(&self) -> Vec<Value> {
        self.executor.stack()
    }
}

impl Default for Tardi {
    fn default() -> Tardi {
        let environment = Environment::with_builtins();
        let scanner = Scanner::default();
        let compiler = Compiler::default();
        let executor = VM::new();
        Tardi::new(environment, scanner, compiler, executor)
    }
}

#[cfg(test)]
mod tests;
