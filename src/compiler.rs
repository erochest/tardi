use std::convert::TryFrom;

use crate::chunk::Chunk;
use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::scanner::{Token, TokenType};
use crate::value::Value;

// TODO: This should probably take tokens: Vec<Result<ScanError, Token>>
pub fn compile(tokens: Vec<Token>) -> Result<Chunk> {
    let mut compiler = Compiler::new(tokens);

    compiler.source_file()?;

    Ok(compiler.chunk)

    // while current < tokens.len() {
    //     let token = &tokens[current];
    //     match &token.token_type {
    //         TokenType::Integer(number) => {
    //             let constant = chunk.add_constant(Value::Integer(*number));
    //             chunk.push_op_code(OpCode::GetConstant, constant as u8);
    //         }
    //         TokenType::Float(number) => {
    //             let constant = chunk.add_constant(Value::Float(*number));
    //             chunk.push_op_code(OpCode::GetConstant, constant as u8);
    //         }
    //         TokenType::Rational(num, denom) => {
    //             let constant = chunk.add_constant(Value::Rational(Rational64::new(*num, *denom)));
    //             chunk.push_op_code(OpCode::GetConstant, constant as u8);
    //         }
    //         TokenType::Boolean(b) => {
    //             let constant = chunk.add_constant(Value::Boolean(*b));
    //             chunk.push_op_code(OpCode::GetConstant, constant as u8);
    //         }
    //         TokenType::String(string) => {
    //             let constant = chunk.add_constant(Value::String(string.clone()));
    //             chunk.push_op_code(OpCode::GetConstant, constant as u8);
    //         }
    //         TokenType::Plus => {
    //             chunk.code.push(OpCode::Add as u8);
    //         }
    //         TokenType::Minus => {
    //             chunk.code.push(OpCode::Sub as u8);
    //         }
    //         TokenType::Star => {
    //             chunk.code.push(OpCode::Mult as u8);
    //         }
    //         TokenType::Slash => {
    //             chunk.code.push(OpCode::Div as u8);
    //         }
    //         TokenType::Equal => chunk.code.push(OpCode::Equal as u8),
    //         TokenType::BangEqual => {
    //             chunk.code.push(OpCode::Equal as u8);
    //             chunk.code.push(OpCode::Not as u8);
    //         }
    //         TokenType::Less => chunk.code.push(OpCode::Less as u8),
    //         TokenType::Greater => chunk.code.push(OpCode::Greater as u8),
    //         TokenType::LessEqual => {
    //             chunk.code.push(OpCode::Greater as u8);
    //             chunk.code.push(OpCode::Not as u8);
    //         }
    //         TokenType::GreaterEqual => {
    //             chunk.code.push(OpCode::Less as u8);
    //             chunk.code.push(OpCode::Not as u8);
    //         }
    //         TokenType::Bang => chunk.code.push(OpCode::Not as u8),
    //         // TODO: These should probably be a compile time function to
    //         // create a vector and a runtime function to get it and
    //         // clone it.
    //         TokenType::OpenBrace => unimplemented!(),
    //         TokenType::CloseBrace => unimplemented!(),
    //         TokenType::Word(_) => todo!(),
    //         TokenType::OpenParen => todo!(),
    //         TokenType::CloseParen => todo!(),
    //         TokenType::Colon => todo!(),
    //         TokenType::Semicolon => todo!(),
    //         TokenType::LongDash => todo!(),
    //     }
    //     current += 1;
    // }

    // Ok(chunk)
}

#[derive(Debug)]
struct Compiler {
    index: usize,
    previous: Option<Token>,
    current: Option<Token>,
    tokens: Vec<Token>,
    chunk: Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl Compiler {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            index: 0,
            previous: None,
            current: None,
            tokens,
            chunk: Chunk::new(),
            had_error: false,
            panic_mode: false,
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.take();

        loop {
            // TODO: remove this clone
            self.current = Some(self.tokens[self.index].clone());
            self.index += 1;
            break;

            // TODO: report on errors and skip over them
        }
    }

    fn end(&mut self) {
        self.push_return();
    }

    fn source_file(&mut self) -> Result<()> {
        while self.index < self.tokens.len() {
            self.advance();
            self.declaration()?;
        }

        self.end();
        Ok(())
    }

    fn declaration(&mut self) -> Result<()> {
        self.expression()
    }

    fn expression(&mut self) -> Result<()> {
        let current = self.current.as_ref().unwrap();
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
            TokenType::Equal => self.push_op_code(OpCode::Equal),
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
            TokenType::OpenBrace => self.vector()?,
            TokenType::CloseBrace => todo!(),
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

    fn vector(&mut self) -> Result<()> {
        todo!("vector")
    }

    // WRITING BYTECODE

    fn push_byte(&mut self, byte: u8) {
        self.chunk.code.push(byte);
    }

    fn push_op_code(&mut self, op_code: OpCode) {
        self.chunk.code.push(op_code as u8);
    }

    fn push_op_code_arg(&mut self, op_code: OpCode, arg: u8) {
        self.chunk.code.push(op_code as u8);
        self.chunk.code.push(arg);
    }

    fn push_constant(&mut self, constant: Value) -> Result<()> {
        if self.chunk.constants.len() >= u8::MAX as usize {
            return Err(Error::TooManyConstants);
        }

        let index = self.chunk.add_constant(constant);
        self.push_op_code_arg(OpCode::GetConstant, index as u8);

        Ok(())
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
        Compiler::new(vec![])
    }
}

#[cfg(test)]
mod tests;
