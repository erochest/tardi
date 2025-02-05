use crate::chunk::Chunk;
use crate::parser::{Token, TokenType};
use crate::value::Value;
use crate::op_code::OpCode;

pub fn compile(tokens: Vec<Token>) -> Chunk {
    let mut chunk = Chunk::new();
    let mut current = 0;
    
    while current < tokens.len() {
        let token = &tokens[current];
        match &token.token_type {
            TokenType::Integer(number) => {
                let constant = chunk.add_constant(Value::Integer(number));
                chunk.code.push(OpCode::GetConstant as u8);
                chunk.code.push(constant as u8);
            },
            TokenType::String(string) => {
                let constant = chunk.add_constant(Value::String(string.clone()));
                chunk.code.push(OpCode::GetConstant as u8);
                chunk.code.push(constant as u8);
            },
            TokenType::Plus => {
                chunk.code.push(OpCode::Add as u8);
            },
            TokenType::Minus => {
                chunk.code.push(OpCode::Sub as u8);
            },
            TokenType::Multiply => {
                chunk.code.push(OpCode::Mult as u8);
            },
            TokenType::Division => {
                chunk.code.push(OpCode::Div as u8);
            },
        }
        current += 1;
    }
    
    chunk
} 

#[cfg(test)]
mod tests;
