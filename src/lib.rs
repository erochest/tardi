//! Tardi environmentming language implementation

pub mod compiler;
pub mod env;
pub mod error;
pub mod scanner;
pub mod shared;
pub mod vm;

use std::io::{self, Write};
use std::path::PathBuf;

// Re-exports
use crate::shared::{shared, Shared};
pub use compiler::Compiler;
pub use env::Environment;
pub use error::Result;
pub use scanner::Scanner;
use scanner::Token;
pub use vm::value::Value;
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
    tardi.execute_str(source)?;
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

        tardi.execute_str(input)?;
        for value in tardi.stack() {
            println!("{}", value);
        }
    }

    Ok(())
}

pub trait Scan {
    fn scan(&mut self, input: &str) -> Vec<Result<Token>>;
}

pub trait Compile {
    fn compile(&mut self, env: Shared<Environment>, tokens: Vec<Result<Token>>) -> Result<()>;
}

pub trait Execute {
    fn run(&mut self, env: Shared<Environment>) -> Result<()>;
    fn stack(&self) -> Vec<Value>;
}

pub struct Tardi {
    input: Option<String>,
    environment: Shared<Environment>,
    scanner: Box<dyn Scan>,
    compiler: Box<dyn Compile>,
    executor: Box<dyn Execute>,
}

impl Tardi {
    pub fn new(
        environment: Environment,
        scanner: Box<dyn Scan>,
        compiler: Box<dyn Compile>,
        executor: Box<dyn Execute>,
    ) -> Self {
        Tardi {
            input: None,
            environment: shared(environment),
            scanner,
            compiler,
            executor,
        }
    }

    pub fn execute_str(&mut self, input: String) -> Result<()> {
        log::debug!("input : {:?}", input);
        self.input = Some(input);
        let tokens = self.scanner.scan(self.input.as_ref().unwrap());

        log::debug!("tokens: {:?}", tokens);
        self.compiler.compile(self.environment.clone(), tokens)?;

        log::debug!("environment:\n{:?}", self.environment.borrow());
        self.executor.run(self.environment.clone())?;

        Ok(())
    }

    pub fn stack(&self) -> Vec<Value> {
        self.executor.stack()
    }
}

impl Default for Tardi {
    fn default() -> Tardi {
        let environment = Environment::with_builtins();
        let scanner = Box::new(Scanner::default());
        let compiler = Box::new(Compiler::default());
        let executor = Box::new(VM::new());
        Tardi::new(environment, scanner, compiler, executor)
    }
}

#[cfg(test)]
mod tests;
