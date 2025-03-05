use crate::chunk::Chunk;
use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::value::Value;
use std::convert::TryFrom;

#[derive(Default)]
pub struct VM {
    pub ip: usize,
    pub stack: Vec<Value>,
    pub call_stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            stack: Vec::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, chunk: &mut Chunk) -> Result<()> {
        self.ip = 0;

        log::trace!("executing chunk {:?}", chunk);

        while self.ip < chunk.code.len() {
            let instruction = chunk.code[self.ip];
            let op = OpCode::try_from(instruction)?;

            if log::log_enabled!(log::Level::Trace) {
                let mut buffer = String::new();
                chunk.debug_op(&mut buffer, &op, self.ip)?;
                log::trace!("executing op: {}", buffer.trim_end());
            }

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
                    self.call_stack.push(Value::from(self.ip + 2));
                    self.ip += 1;
                    self.ip = chunk.code[self.ip] as usize;
                    log::trace!("MARK-JUMP@{}: moving to ip {}", ip, self.ip);
                    continue;
                }
                OpCode::MarkCall => todo!(),
                OpCode::CallTardiFn => {
                    self.ip += 1;
                    let index = chunk.code[self.ip] as usize;
                    let tardi_fn = &mut chunk.builtins[index];
                    tardi_fn.call(self)?;
                }
                OpCode::ToCallStack => {
                    let item = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.call_stack.push(item);
                }
                OpCode::FromCallStack => {
                    let item = self.call_stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push(item);
                }
                OpCode::CopyCallStack => {
                    let item = self.call_stack.last().ok_or(Error::StackUnderflow)?;
                    self.stack.push(item.clone());
                }
                OpCode::Return => {
                    let ip = self.ip;
                    if let Some(Value::Address(return_ip)) = self.call_stack.pop() {
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

#[cfg(test)]
mod tests;
