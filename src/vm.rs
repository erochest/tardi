use crate::chunk::Chunk;
use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::value::Value;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;

type SharedValue = Rc<RefCell<Value>>;

#[macro_export]
macro_rules! binary_op {
    ($vm:expr, $op:tt) => {
        {
            let b = $vm.stack.pop().ok_or(Error::StackUnderflow)?;
            let a = $vm.stack.pop().ok_or(Error::StackUnderflow)?;
            let b = b.borrow();
            let a = a.borrow();
            let c = (a.clone() $op b.clone())?;
            let c = shared(c);
            $vm.stack.push(c);
        }
    };
}

#[macro_export]
macro_rules! pop_unwrap {
    ($stack:expr) => {{
        let a = $stack.pop().ok_or(Error::StackUnderflow)?;
        let a = a.borrow();
        let a = a.clone();
        a
    }};
}

#[derive(Default)]
pub struct VM {
    pub ip: usize,
    pub stack: Vec<SharedValue>,
    pub call_stack: Vec<SharedValue>,
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
                    let constant = shared(constant);
                    self.stack.push(constant);
                }
                OpCode::Add => binary_op!(self, +),
                OpCode::Sub => binary_op!(self, -),
                OpCode::Mult => binary_op!(self, *),
                OpCode::Div => binary_op!(self, /),
                OpCode::Modulo => binary_op!(self, %),
                OpCode::Equal => {
                    let b = pop_unwrap!(self.stack);
                    let a = pop_unwrap!(self.stack);
                    let c = shared(Value::Boolean(a == b));
                    self.stack.push(c);
                }
                OpCode::Not => {
                    // TODO: add better type checking. this just assumes it's a boolean
                    let a = pop_unwrap!(self.stack);
                    match a {
                        Value::Boolean(a) => {
                            self.stack.push(shared(Value::Boolean(!a)));
                        }
                        v => {
                            let error_v = v.clone();
                            self.stack.push(shared(v));
                            return Err(Error::InvalidValueType(error_v));
                        }
                    }
                }
                OpCode::Less => {
                    let b = pop_unwrap!(self.stack);
                    let a = pop_unwrap!(self.stack);
                    self.stack.push(shared(Value::Boolean(a < b)));
                }
                OpCode::Greater => {
                    let b = pop_unwrap!(self.stack);
                    let a = pop_unwrap!(self.stack);
                    self.stack.push(shared(Value::Boolean(a > b)));
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
                    self.call_stack.push(shared(Value::from(self.ip + 2)));
                    self.ip += 1;
                    self.ip = chunk.code[self.ip] as usize;
                    log::trace!("MARK-JUMP@{}: moving to ip {}", ip, self.ip);
                    continue;
                }
                OpCode::CallTardiFn => {
                    self.ip += 1;
                    let index = chunk.code[self.ip] as usize;
                    let mut tardi_fn = chunk.builtins[index].clone();
                    tardi_fn.call(self, chunk)?;
                }
                OpCode::ToCallStack => {
                    let item = pop_unwrap!(self.stack);
                    self.call_stack.push(shared(item));
                }
                OpCode::FromCallStack => {
                    let item = self.call_stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push(item);
                }
                OpCode::CopyCallStack => {
                    let item = self.call_stack.last().ok_or(Error::StackUnderflow)?;
                    self.stack.push(item.clone());
                }
                OpCode::Drop => {
                    let _ = self.stack.pop().ok_or(Error::StackUnderflow)?;
                }
                OpCode::Swap => {
                    let b = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(Error::StackUnderflow)?;
                    self.stack.push(b);
                    self.stack.push(a);
                }
                OpCode::IP => {
                    let next_ip = self.ip + 1;
                    self.stack.push(shared(Value::Address(next_ip)));
                }
                OpCode::Return => {
                    let ip = self.ip;
                    if !self.call_stack.is_empty() {
                        let a = pop_unwrap!(self.call_stack);
                        if let Value::Address(return_ip) = a {
                            log::trace!("RETURN@{}: {}", ip, return_ip);
                            self.ip = return_ip;
                            continue;
                        } else {
                            return Err(Error::InvalidValueType(a.clone()));
                        }
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
            eprintln!("{}", value.borrow());
        }
    }
}

pub fn shared(value: Value) -> SharedValue {
    Rc::new(RefCell::new(value))
}

#[cfg(test)]
mod tests;
