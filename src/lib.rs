//! Tardi environmentming language implementation

/// Re-export error types
pub mod error;

/// Virtual Machine implementation
pub mod vm;

/// Scanner implementation
pub mod scanner;

/// Compiler implementation
pub mod compiler;

pub mod env;

use std::path::PathBuf;

// Re-exports
pub use compiler::Compiler;
pub use env::Environment;
pub use error::Result;
pub use scanner::Scanner;
use scanner::Token;
use vm::create_op_table;
pub use vm::value::Value;
use vm::value::{shared, Shared};
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

    // let scanner = Scanner::new(&source);
    // let mut compiler = Compiler::new();
    // let environment = compiler.compile(scanner)?;

    // let mut vm = VM::new();
    // vm.load_environment(Box::new(environment));
    // vm.run()?;

    if print_stack {
        // Print stack contents from top to bottom
        for value in tardi.stack() {
            eprintln!("{}", value);
        }
    }

    Ok(())
}

pub trait Scan {
    fn scan(&mut self, input: &str) -> Vec<Result<Token>>;
}

pub trait Compile {
    fn compile(&mut self, tokens: Vec<Result<Token>>) -> Result<Environment>;
}

pub trait Execute {
    fn run(&mut self) -> Result<()>;
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
        environment: Shared<Environment>,
        scanner: Box<dyn Scan>,
        compiler: Box<dyn Compile>,
        executor: Box<dyn Execute>,
    ) -> Self {
        // TODO: this doesn't feel like the right place for this.
        environment.borrow_mut().set_op_table(create_op_table());
        Tardi {
            input: None,
            environment,
            scanner,
            compiler,
            executor,
        }
    }

    pub fn execute_str(&mut self, input: String) -> Result<()> {
        // TODO: have everything work on the one Environment created first
        self.input = Some(input);
        let tokens = self.scanner.scan(self.input.as_ref().unwrap());
        // TODO: how does Compile have access to the environment?
        *self.environment.borrow_mut() = self.compiler.compile(tokens)?;
        self.executor.run()?;
        Ok(())
    }

    pub fn stack(&self) -> Vec<Value> {
        self.executor.stack()
    }
}

impl Default for Tardi {
    fn default() -> Tardi {
        let environment = shared(Environment::default());
        let scanner = Box::new(Scanner::default());
        let compiler = Box::new(Compiler::default());
        let executor = Box::new(VM::new(environment.clone()));
        Tardi::new(environment, scanner, compiler, executor)
    }
}

#[cfg(test)]
mod tests;
