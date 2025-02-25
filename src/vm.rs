use crate::chunk::Chunk;
use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::value::Value;
use std::convert::TryFrom;

#[derive(Default)]
pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    call_stack: Vec<Return>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            stack: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, chunk: Chunk) -> Result<()> {
        while self.ip < chunk.code.len() {
            let instruction = chunk.code[self.ip];
            let op = OpCode::try_from(instruction)?;

            log::trace!("executing instruction: {:?}", op);

            // TODO: on errors, need to restore the stack
            match op {
                OpCode::GetConstant => {
                    self.ip += 1;
                    let constant_idx = chunk.code[self.ip];
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
                OpCode::Jump => {
                    let ip = self.ip;
                    self.ip += 1;
                    self.ip = chunk.code[self.ip] as usize;
                    log::trace!("JUMP@{}: moving to ip {}", ip, self.ip);
                    continue;
                }
                OpCode::MarkJump => {
                    let ip = self.ip;
                    log::trace!("MARK-JUMP@{}: call-stack pushing ip {}", ip, self.ip + 2);
                    self.call_stack.push(Return::new(self.ip + 2));
                    self.ip += 1;
                    self.ip = chunk.code[self.ip] as usize;
                    log::trace!("MARK-JUMP@{}: moving to ip {}", ip, self.ip);
                    continue;
                }
                OpCode::Return => {
                    let ip = self.ip;
                    if let Some(Return { ip: return_ip }) = self.call_stack.pop() {
                        log::trace!("RETURN@{}: {}", ip, return_ip);
                        self.ip = return_ip;
                        continue;
                    } else {
                        log::trace!("RETURN@{}: ALL", ip);
                        break;
                    }
                }
            }

            self.ip += 1;
        }

        Ok(())
    }

    pub fn print_stack(&self) {
        for value in &self.stack {
            eprintln!("{}", value);
        }
    }
}

#[derive(Debug, Default)]
struct Return {
    ip: usize,
}

impl Return {
    fn new(ip: usize) -> Self {
        Self { ip }
    }
}

#[cfg(test)]
mod tests;
