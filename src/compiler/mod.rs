mod program;

pub use self::program::Program;
use crate::error::{CompilerError, Error, Result};
use crate::scanner::{Token, TokenType};
use crate::vm::create_op_table;
use crate::vm::value::Value;

pub struct Compiler {
    program: Program,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        let mut program = Program::new();
        let (op_table, op_map) = create_op_table();
        program.set_op_table(op_table, op_map);

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
        // TODO: remove the catch-all branch once the dust has settled
        match token.token_type {
            TokenType::Integer(value) => self.compile_integer(value),
            TokenType::Float(value) => self.compile_float(value),
            TokenType::Boolean(value) => self.compile_boolean(value),
            TokenType::Char(value) => self.compile_char(value),
            TokenType::Dup => self.compile_op("dup"),
            TokenType::Swap => self.compile_op("swap"),
            TokenType::Rot => self.compile_op("rot"),
            TokenType::Drop => self.compile_op("drop"),
            TokenType::ToR => self.compile_op(">r"),
            TokenType::RFrom => self.compile_op("r>"),
            TokenType::RFetch => self.compile_op("r@"),
            TokenType::Plus => self.compile_op("+"),
            TokenType::Dash => self.compile_op("-"),
            TokenType::Star => self.compile_op("*"),
            TokenType::Slash => self.compile_op("/"),
            TokenType::EqualEqual => self.compile_op("=="),
            TokenType::BangEqual => {
                self.compile_op("==")?;
                self.compile_op("!")
            }
            TokenType::Less => self.compile_op("<"),
            TokenType::Greater => self.compile_op(">"),
            TokenType::LessEqual => {
                self.compile_op(">")?;
                self.compile_op("!")
            }
            TokenType::GreaterEqual => {
                self.compile_op("<")?;
                self.compile_op("!")
            }
            TokenType::Bang => self.compile_op("!"),
            TokenType::Word(word) => Err(Error::CompilerError(CompilerError::UnsupportedToken(
                format!("word: {}", word),
            ))),
            _ => Err(Error::CompilerError(CompilerError::UnsupportedToken(
                format!("{:?}", token),
            ))),
        }
    }

    fn compile_integer(&mut self, value: i64) -> Result<()> {
        let const_index = self.program.add_constant(Value::Integer(value));
        let lit_index = self
            .program
            .get_op_index("lit")
            .ok_or(Error::CompilerError(CompilerError::InvalidOperation(
                "lit operation not found".to_string(),
            )))?;
        self.program.add_instruction(lit_index);
        self.program.add_instruction(const_index);
        Ok(())
    }

    fn compile_float(&mut self, value: f64) -> Result<()> {
        let const_index = self.program.add_constant(Value::Float(value));
        let lit_index = self
            .program
            .get_op_index("lit")
            .ok_or(Error::CompilerError(CompilerError::InvalidOperation(
                "lit operation not found".to_string(),
            )))?;
        self.program.add_instruction(lit_index);
        self.program.add_instruction(const_index);
        Ok(())
    }

    fn compile_boolean(&mut self, value: bool) -> Result<()> {
        let const_index = self.program.add_constant(Value::Boolean(value));
        let lit_index = self
            .program
            .get_op_index("lit")
            .ok_or(Error::CompilerError(CompilerError::InvalidOperation(
                "lit operation not found".to_string(),
            )))?;
        self.program.add_instruction(lit_index);
        self.program.add_instruction(const_index);
        Ok(())
    }

    fn compile_char(&mut self, value: char) -> Result<()> {
        let const_index = self.program.add_constant(Value::Char(value));
        let lit_index = self
            .program
            .get_op_index("lit")
            .ok_or(Error::CompilerError(CompilerError::InvalidOperation(
                "lit operation not found".to_string(),
            )))?;
        self.program.add_instruction(lit_index);
        self.program.add_instruction(const_index);
        Ok(())
    }

    fn compile_op(&mut self, op_name: &str) -> Result<()> {
        let op_index = self
            .program
            .get_op_index(op_name)
            .ok_or(Error::CompilerError(CompilerError::InvalidOperation(
                format!("{} operation not found", op_name),
            )))?;
        self.program.add_instruction(op_index);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
