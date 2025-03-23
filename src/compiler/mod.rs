mod program;

pub use self::program::Program;
use crate::error::{CompilerError, Error, Result};
use crate::scanner::{Token, TokenType};
use crate::vm::{create_op_table, OpCode};
use crate::vm::value::Value;

pub struct Compiler {
    program: Program,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// TODO:
// Methods to add:
// - start_callable
// - complete_callable

impl Compiler {
    pub fn new() -> Self {
        let mut program = Program::new();
        let op_table = create_op_table();
        program.set_op_table(op_table);

        Compiler { program }
    }

    pub fn compile(&mut self, tokens: impl Iterator<Item = Result<Token>>) -> Result<Program> {
        for token_result in tokens {
            let token = token_result?;
            self.compile_token(token)?;
        }
        Ok(self.program.clone())
    }

    fn compile_token(&mut self, token: Token) -> Result<()> {
        match token.token_type {
            TokenType::Integer(value) => self.compile_constant(value),
            TokenType::Float(value) => self.compile_constant(value),
            TokenType::Boolean(value) => self.compile_constant(value),
            TokenType::Char(value) => self.compile_constant(value),
            TokenType::Dup => self.compile_op(OpCode::Dup),
            TokenType::Swap => self.compile_op(OpCode::Swap),
            TokenType::Rot => self.compile_op(OpCode::Rot),
            TokenType::Drop => self.compile_op(OpCode::Drop),
            TokenType::ToR => self.compile_op(OpCode::ToR),
            TokenType::RFrom => self.compile_op(OpCode::RFrom),
            TokenType::RFetch => self.compile_op(OpCode::RFetch),
            TokenType::Plus => self.compile_op(OpCode::Add),
            TokenType::Dash => self.compile_op(OpCode::Subtract),
            TokenType::Star => self.compile_op(OpCode::Multiply),
            TokenType::Slash => self.compile_op(OpCode::Divide),
            TokenType::EqualEqual => self.compile_op(OpCode::Equal),
            TokenType::BangEqual => {
                self.compile_op(OpCode::Equal)?;
                self.compile_op(OpCode::Not)
            }
            TokenType::Less => self.compile_op(OpCode::Less),
            TokenType::Greater => self.compile_op(OpCode::Greater),
            TokenType::LessEqual => {
                self.compile_op(OpCode::Greater)?;
                self.compile_op(OpCode::Not)
            }
            TokenType::GreaterEqual => {
                self.compile_op(OpCode::Less)?;
                self.compile_op(OpCode::Not)
            }
            TokenType::Bang => self.compile_op(OpCode::Not),
            TokenType::CreateList => self.compile_op(OpCode::CreateList),
            TokenType::Append => self.compile_op(OpCode::Append),
            TokenType::Prepend => self.compile_op(OpCode::Prepend),
            TokenType::Concat => self.compile_op(OpCode::Concat),
            TokenType::SplitHead => self.compile_op(OpCode::SplitHead),
            TokenType::String(value) => self.compile_constant(value),
            TokenType::CreateString => self.compile_op(OpCode::CreateString),
            TokenType::ToString => self.compile_op(OpCode::ToString),
            TokenType::Utf8ToString => self.compile_op(OpCode::Utf8ToString),
            TokenType::StringConcat => self.compile_op(OpCode::StringConcat),
            TokenType::Word(word) => Err(Error::CompilerError(CompilerError::UnsupportedToken(
                format!("word: {}", word),
            ))),
            _ => Err(Error::CompilerError(CompilerError::UnsupportedToken(
                format!("{:?}", token),
            ))),
        }
    }

    fn compile_constant<T: Into<Value>>(&mut self, value: T) -> Result<()> {
        let const_index = self.program.add_constant(value.into());
        self.program.add_op_arg(OpCode::Lit, const_index);
        Ok(())
    }

    fn compile_op(&mut self, op: OpCode) -> Result<()> {
        self.program.add_op(op);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
