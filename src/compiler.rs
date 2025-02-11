use crate::chunk::Chunk;
use crate::op_code::OpCode;
use crate::parser::{Token, TokenType};
use crate::value::Value;

pub fn compile(tokens: Vec<Token>) -> Chunk {
    let mut chunk = Chunk::new();
    let mut current = 0;

    while current < tokens.len() {
        let token = &tokens[current];
        match &token.token_type {
            TokenType::Integer(number) => {
                let constant = chunk.add_constant(Value::Integer(*number));
                chunk.push_op_code(OpCode::GetConstant, constant as u8);
            }
            TokenType::Float(number) => {
                let constant = chunk.add_constant(Value::Float(*number));
                chunk.push_op_code(OpCode::GetConstant, constant as u8);
            }
            TokenType::Rational(num, denom) => {
                let constant = chunk.add_constant(Value::Rational(*num, *denom));
                chunk.push_op_code(OpCode::GetConstant, constant as u8);
            }
            TokenType::String(string) => {
                let constant = chunk.add_constant(Value::String(string.clone()));
                chunk.push_op_code(OpCode::GetConstant, constant as u8);
            }
            TokenType::Plus => {
                chunk.code.push(OpCode::Add as u8);
            }
            TokenType::Minus => {
                chunk.code.push(OpCode::Sub as u8);
            }
            TokenType::Multiply => {
                chunk.code.push(OpCode::Mult as u8);
            }
            TokenType::Division => {
                chunk.code.push(OpCode::Div as u8);
            }
        }
        current += 1;
    }

    chunk
}

#[cfg(test)]
mod tests;
