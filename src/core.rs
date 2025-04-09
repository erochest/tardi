use crate::env::Environment;
use crate::error::Result;
use crate::scanner::{Token, TokenType};
use crate::shared::Shared;
use crate::value::{Function, Value};

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
