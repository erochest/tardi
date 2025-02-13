use crate::chunk::Chunk;
use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::value::Value;
use std::convert::TryFrom;

#[derive(Default)]
pub struct VM {
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM { stack: Vec::new() }
    }

    pub fn execute(&mut self, chunk: Chunk) -> Result<()> {
        let mut ip = 0;

        while ip < chunk.code.len() {
            let instruction = chunk.code[ip];

            // TODO: on errors, need to restore the stack
            match OpCode::try_from(instruction)? {
                OpCode::GetConstant => {
                    ip += 1;
                    let constant_idx = chunk.code[ip];
                    let constant = chunk.constants[constant_idx as usize].clone();
                    self.stack.push(constant);
                }
                OpCode::Add => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push((a + b)?);
                }
                OpCode::Sub => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push((a - b)?);
                }
                OpCode::Mult => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push((a * b)?);
                }
                OpCode::Div => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push(a.checked_div(b)?);
                }
                OpCode::Equal => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Not => match self.stack.pop() {
                    Some(Value::Boolean(a)) => {
                        self.stack.push(Value::Boolean(!a));
                    }
                    Some(v) => {
                        let error_v = v.clone();
                        self.stack.push(v);
                        return Err(Error::InvalidValueType(error_v));
                    }
                    _ => {
                        return Err(Error::StackUnderflow);
                    }
                },
                OpCode::Less => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push(Value::Boolean(a < b));
                }
                OpCode::Greater => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push(Value::Boolean(a > b));
                }
            }

            ip += 1;
        }

        Ok(())
    }

    pub fn print_stack(&self) {
        for value in &self.stack {
            eprintln!("{}", value);
        }
    }
}

#[cfg(test)]
mod tests;
