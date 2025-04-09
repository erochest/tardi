use crate::compiler::Compiler;
use crate::env::Environment;
use crate::error::Result;
use crate::scanner::{Scanner, Token, TokenType};
use crate::shared::{shared, Shared};
use crate::value::{Callable, Function, Value};
use crate::vm::VM;

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
    pub input: Option<String>,
    pub environment: Shared<Environment>,
    pub scanner: Scanner,
    pub compiler: Compiler,
    pub executor: VM,
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
        self.inject_macro_readers()?;
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

    fn inject_macro_readers(&mut self) -> Result<()> {
        let env = self.environment.borrow_mut();
        let op_map = env.get_op_map();

        // if !op_map.contains_key("scan-token") {
        //     let callable = Callable::Fn(todo!("scan-token implementation"));
        //     env.add_to_op_table(shared(callable));
        // }

        Ok(())
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
