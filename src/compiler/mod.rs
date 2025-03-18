mod program;

pub use self::program::Program;
use crate::error::{CompilerError, Error, Result};
use crate::scanner::{Token, TokenType};
use crate::vm::{create_op_table, Value};

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
            TokenType::Dup => self.compile_op("dup"),
            TokenType::Swap => self.compile_op("swap"),
            TokenType::Rot => self.compile_op("rot"),
            TokenType::Drop => self.compile_op("drop"),
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
mod tests {
    use super::*;
    use crate::compiler::program::Program;
    use crate::scanner::Scanner;

    use pretty_assertions::assert_eq;

    // TODO: more tests

    fn compile(input: &str) -> Result<Program> {
        let scanner = Scanner::new(input);
        let mut compiler = Compiler::new();
        compiler.compile(scanner)
    }

    #[test]
    fn test_compile_comparison_operators() -> Result<()> {
        let program = compile("1 2 == 3 4 != 5 6 < 7 8 > 9 10 <= 11 12 >=")?;

        let expected_ops = vec![
            "lit", "lit", "==", // 1 2 ==
            "lit", "lit", "==", "!", // 3 4 != (implemented as == !)
            "lit", "lit", "<", // 5 6 <
            "lit", "lit", ">", // 7 8 >
            "lit", "lit", ">", "!", // 9 10 <= (implemented as > !)
            "lit", "lit", "<", "!", // 11 12 >= (implemented as < !)
        ];
        let mut actual_ops = Vec::new();
        let instructions = program.get_instructions();
        let mut i = 0;
        while i < instructions.len() {
            let op = instructions[i];
            let name = program.get_op_name(op).unwrap().to_string();
            actual_ops.push(name.clone());
            if name == "lit" {
                i += 2;
            } else {
                i += 1;
            }
        }

        assert_eq!(actual_ops, expected_ops);
        Ok(())
    }

    #[test]
    fn test_compile_word() -> Result<()> {
        let result = compile("custom_word");
        assert!(result.is_err());
        if let Err(Error::CompilerError(CompilerError::UnsupportedToken(msg))) = result {
            assert_eq!(msg, "word: custom_word");
        } else {
            panic!("Expected UnsupportedToken error");
        }
        Ok(())
    }
}
