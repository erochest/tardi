use std::convert::TryFrom;

use crate::chunk::Chunk;
use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

// TODO: This should probably take tokens: Vec<Result<ScanError, Token>>
pub fn compile(input: &str) -> Result<Chunk> {
    let scanner = Scanner::from_string(input);
    let mut compiler = Compiler::new(scanner);

    compiler.source_file()?;

    Ok(compiler.chunk)
}

#[derive(Debug)]
struct Compiler {
    scanner: Scanner,
    previous: Option<Token>,
    current: Option<Token>,
    chunk: Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl Compiler {
    fn new(scanner: Scanner) -> Self {
        Self {
            scanner,
            previous: None,
            current: None,
            chunk: Chunk::new(),
            had_error: false,
            panic_mode: false,
        }
    }

    /// Scan the next token, skipping over any error tokens.
    ///
    /// At the end of this,
    /// - self.current is the new token
    /// - self.previous is the token that was previously current
    fn advance(&mut self) {
        self.previous = self.current.take();

        loop {
            match self.scanner.next() {
                Ok(current) => {
                    self.current = current;
                    break;
                }
                Err(err) => todo!("error handling: {:?}", err),
            }
        }
    }

    fn end(&mut self) {
        self.push_return();
    }

    fn at_end(&self) -> bool {
        self.current.is_none()
    }

    fn source_file(&mut self) -> Result<()> {
        self.advance();
        while !self.at_end() {
            self.declaration()?;
            self.advance();
        }

        self.end();
        Ok(())
    }

    fn declaration(&mut self) -> Result<()> {
        self.expression()
    }

    fn expression(&mut self) -> Result<()> {
        let current = self.current.as_ref().unwrap();
        // TODO: this is too much. you shouldn't be able to define a function here.
        // TODO: how to use `literal` here? and then the rest of it?
        match current.token_type {
            TokenType::Integer(_)
            | TokenType::Float(_)
            | TokenType::Rational(_, _)
            | TokenType::String(_)
            | TokenType::Boolean(_) => {
                // TODO: de-clone
                let value = Value::try_from(current.token_type.clone())?;
                self.push_constant(value)?;
            }
            TokenType::Word(_) => todo!(),
            TokenType::Plus => self.push_op_code(OpCode::Add),
            TokenType::Minus => self.push_op_code(OpCode::Sub),
            TokenType::Star => self.push_op_code(OpCode::Mult),
            TokenType::Slash => self.push_op_code(OpCode::Div),
            TokenType::EqualEqual => self.push_op_code(OpCode::Equal),
            TokenType::BangEqual => {
                self.push_op_code(OpCode::Equal);
                self.push_op_code(OpCode::Not);
            }
            TokenType::Less => self.push_op_code(OpCode::Less),
            TokenType::Greater => self.push_op_code(OpCode::Greater),
            TokenType::LessEqual => {
                self.push_op_code(OpCode::Greater);
                self.push_op_code(OpCode::Not);
            }
            TokenType::GreaterEqual => {
                self.push_op_code(OpCode::Less);
                self.push_op_code(OpCode::Not);
            }
            TokenType::Bang => self.push_op_code(OpCode::Not),
            TokenType::OpenBrace => {
                self.vector(true)?;
            }
            TokenType::CloseBrace => unimplemented!(),
            TokenType::OpenParen => todo!(),
            TokenType::CloseParen => todo!(),
            TokenType::Colon => self.function()?,
            TokenType::Semicolon => todo!(),
            TokenType::LongDash => todo!(),
            TokenType::EOF => {}
        }

        Ok(())
    }

    fn function(&mut self) -> Result<()> {
        todo!("function")
    }

    fn literal(&mut self, with_stack_ops: bool) -> Result<u8> {
        let current = self.current.as_ref().unwrap();
        match current.token_type {
            TokenType::Integer(_)
            | TokenType::Float(_)
            | TokenType::Rational(_, _)
            | TokenType::String(_)
            | TokenType::Boolean(_) => {
                // TODO: de-clone
                let value = Value::try_from(current.token_type.clone())?;
                let index = self.add_constant(value)?;
                if with_stack_ops {
                    self.push_op_code_arg(OpCode::GetConstant, index);
                }
                Ok(index)
            }
            TokenType::OpenBrace => self.vector(with_stack_ops),
            _ => return Err(Error::PrecedenceError),
        }
    }

    fn vector(&mut self, with_stack_ops: bool) -> Result<u8> {
        let mut vector = Vec::new();

        self.advance();
        while !self.at_end()
            && self
                .current
                .as_ref()
                .map(|t| t.token_type != TokenType::CloseBrace)
                .unwrap_or(false)
        {
            let index = self.literal(false)?;
            let value = self.chunk.constants[index as usize].clone();
            vector.push(value);

            self.advance();
        }

        let vector = Value::Vector(vector);
        let index = self.add_constant(vector)?;
        if with_stack_ops {
            self.push_op_code_arg(OpCode::GetConstant, index);
        }

        Ok(index)
    }

    // WRITING BYTECODE

    /// This pushes a byet onto the chunk's code block.
    fn push_byte(&mut self, byte: u8) {
        self.chunk.code.push(byte);
    }

    /// This pushes an op code onto the chunk's code black.
    fn push_op_code(&mut self, op_code: OpCode) {
        self.push_byte(op_code as u8);
    }

    /// This pushes an op code and it's argument onte the chunk's code block.
    fn push_op_code_arg(&mut self, op_code: OpCode, arg: u8) {
        self.push_op_code(op_code);
        self.push_byte(arg);
    }

    /// This allocates a constant and emits the op code to push it onto the stack.
    fn push_constant(&mut self, constant: Value) -> Result<()> {
        let index = self.add_constant(constant)?;
        self.push_op_code_arg(OpCode::GetConstant, index);
        Ok(())
    }

    /// This allocates a constant and returns its index in the constant table.
    fn add_constant(&mut self, constant: Value) -> Result<u8> {
        if self.chunk.constants.len() >= u8::MAX as usize {
            return Err(Error::TooManyConstants);
        }
        let index = self.chunk.add_constant(constant);
        Ok(index as u8)
    }

    fn push_return(&mut self) {
        self.push_op_code(OpCode::Return);
    }

    // ERROR HANDLING

    fn error_at_current(&mut self, message: &str) {
        let token = self.current.as_ref().unwrap().clone();
        self.error_at(&token, message);
    }

    fn error(&mut self, message: &str) {
        let token = self.previous.as_ref().unwrap().clone();
        self.error_at(&token, message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        // TODO: can probably do this better
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        eprintln!(
            "[line {} @ {}] Error {:?}: {}",
            token.line_no, token.column, token.token_type, message
        );

        self.had_error = true;
    }

    // ACCESS UTILITIES

    fn get_current_token_type(&self) -> Option<&TokenType> {
        self.current.as_ref().map(|t| &t.token_type)
    }
}

macro_rules! consume {
    ($compiler:expr, $token_type_match:tt, $message:expr) => {
        if matches!($compiler.current, Some($token_type_match)) {
            $compiler.advance();
        } else {
            $compiler.error_at_current($message);
        }
    };
}

impl Default for Compiler {
    fn default() -> Self {
        Compiler::new(Scanner::from_string(""))
    }
}

#[cfg(test)]
mod tests;
