use crate::shared::{shared, Shared};
use crate::{Compiler, Scanner};
use log::{log_enabled, Level};
use std::fmt::Debug;

use crate::env::{EnvLoc, Environment};
use crate::error::{Error, Result, VMError};
use crate::scanner::TokenType;

pub mod ops;
pub use self::ops::OpCode;

use crate::value::{Callable, Function, SharedValue, Value};

use crate::core::Execute;

// TODO: change `OpFn` not to take any parameters (will this work with methods?)
// TODO: define builtins for `scan-token`, `scan-token-list`, and `scan-value-list`
/// Function pointer type for VM operations
pub type OpFn = fn(&mut VM) -> Result<()>;

/// The Virtual Machine implementation using Indirect Threaded Code (ITC)
pub struct VM {
    /// The environment being executed
    pub environment: Option<Shared<Environment>>,

    /// Instruction pointer tracking the current position in the instruction stream
    pub ip: usize,

    /// Data stack for operation arguments and results
    pub stack: Vec<SharedValue>,

    /// Return stack for control flow
    pub return_stack: Vec<SharedValue>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    /// Creates a new VM instance
    pub fn new() -> Self {
        VM {
            environment: None,
            ip: 0,
            stack: Vec::new(),
            return_stack: Vec::new(),
        }
    }

    /// Pushes a shared value onto the return stack
    pub fn push_return(&mut self, value: SharedValue) -> Result<()> {
        if self.return_stack.len() >= 1024 {
            return Err(VMError::ReturnStackOverflow.into());
        }
        self.return_stack.push(value);
        Ok(())
    }

    /// Pops a shared value from the return stack
    pub fn pop_return(&mut self) -> Result<SharedValue> {
        self.return_stack
            .pop()
            .ok_or(VMError::ReturnStackUnderflow.into())
    }

    /// Moves an item from the data stack to the return stack (>r operation)
    pub fn to_r(&mut self) -> Result<()> {
        let value = self.pop()?;
        self.push_return(value)
    }

    /// Moves an item from the return stack to the data stack (r> operation)
    pub fn r_from(&mut self) -> Result<()> {
        let value = self.pop_return()?;
        self.push(value)
    }

    /// Copies the top item from the return stack to the data stack (r@ operation)
    pub fn r_fetch(&mut self) -> Result<()> {
        if let Some(value) = self.return_stack.last() {
            self.push(value.clone())
        } else {
            Err(VMError::ReturnStackUnderflow.into())
        }
    }

    /// Pushes a shared value onto the data stack
    pub fn push(&mut self, value: SharedValue) -> Result<()> {
        if self.stack.len() >= 1024 {
            return Err(VMError::StackOverflow.into());
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pops a shared value from the data stack
    pub fn pop(&mut self) -> Result<SharedValue> {
        self.stack.pop().ok_or(VMError::StackUnderflow.into())
    }

    /// Returns the current size of the data stack
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Executes the lit operation - loads a constant onto the stack
    pub fn lit(&mut self) -> Result<()> {
        // TODO: Seems like could combine the const_index and constant/value pipelines here
        let const_index = self
            .environment
            .as_ref()
            .and_then(|e| e.borrow().get_instruction(self.ip))
            .ok_or(Error::VMError(VMError::InvalidInstructionPointer(self.ip)))?;
        self.ip += 1;

        let value = {
            let constant = self
                .environment
                .as_ref()
                .and_then(|e| e.borrow().get_constant(const_index).cloned());
            if let Some(value) = constant {
                shared(value.clone())
            } else {
                return Err(Error::VMError(VMError::InvalidConstantIndex(const_index)));
            }
        };
        self.push(value)?;

        Ok(())
    }

    /// Duplicates the top item on the stack
    pub fn dup(&mut self) -> Result<()> {
        let value = self.pop()?;
        self.push(value.clone())?;
        self.push(value)
    }

    /// Swaps the top two items on the stack
    pub fn swap(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(b)?;
        self.push(a)
    }

    /// Rotates the top three items on the stack
    pub fn rot(&mut self) -> Result<()> {
        let c = self.pop()?;
        let b = self.pop()?;
        let a = self.pop()?;
        self.push(b)?;
        self.push(c)?;
        self.push(a)
    }

    /// Removes the top item from the stack
    pub fn drop_op(&mut self) -> Result<()> {
        self.pop().map(|_| ())
    }

    /// Returns the current size of the data stack
    pub fn stack_size_op(&mut self) -> Result<()> {
        self.stack
            .push(shared(Value::Integer(self.stack_size() as i64)));
        Ok(())
    }

    /// Adds the top two items on the stack
    pub fn add(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a + b)?;
        self.push(shared(result))
    }

    /// Subtracts the top item from the second item on the stack
    pub fn subtract(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a - b)?;
        self.push(shared(result))
    }

    /// Multiplies the top two items on the stack
    pub fn multiply(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a * b)?;
        self.push(shared(result))
    }

    /// Divides the second item by the top item on the stack
    pub fn divide(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        let result = (a / b)?;
        self.push(shared(result))
    }

    /// Compares if two values are equal
    pub fn equal(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        if a.partial_cmp(&b).is_none() {
            return Err(VMError::TypeMismatch("equality comparison".to_string()).into());
        }
        self.push(shared(Value::Boolean(a == b)))
    }

    /// Compares if a is less than b
    pub fn less(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(Value::Boolean(ordering.is_lt()))),
            None => Err(VMError::TypeMismatch("less than comparison".to_string()).into()),
        }
    }

    /// Compares if a is greater than b
    pub fn greater(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(Value::Boolean(ordering.is_gt()))),
            None => Err(VMError::TypeMismatch("greater than comparison".to_string()).into()),
        }
    }

    /// Performs logical NOT operation on the top stack item
    pub fn not(&mut self) -> Result<()> {
        let value = self.pop()?.borrow().clone();
        match value {
            Value::Boolean(b) => self.push(shared(Value::Boolean(!b))),
            _ => Err(VMError::TypeMismatch("logical NOT".to_string()).into()),
        }
    }

    /// Creates a new empty list and pushes it onto the stack
    pub fn create_list(&mut self) -> Result<()> {
        self.push(shared(Value::List(Vec::new())))
    }

    /// Appends a value to the end of a list
    pub fn append(&mut self) -> Result<()> {
        let list = self.pop()?;
        let value = self.pop()?;

        (*list)
            .borrow_mut()
            .get_list_mut()
            .map(|l| l.push(value))
            .ok_or_else(|| VMError::TypeMismatch("append to list".to_string()))?;

        Ok(())
    }

    /// Prepends a value to the beginning of a list
    pub fn prepend(&mut self) -> Result<()> {
        let list = self.pop()?;
        let value = self.pop()?;

        (*list)
            .borrow_mut()
            .get_list_mut()
            .map(|l| l.insert(0, value))
            .ok_or_else(|| VMError::TypeMismatch("prepend to list".to_string()))?;

        Ok(())
    }

    /// Concatenates two lists
    pub fn concat(&mut self) -> Result<()> {
        let list2 = self.pop()?;
        let list1 = self.pop()?;

        let new_items = {
            let list1_ref = list1.borrow();
            let list2_ref = list2.borrow();
            match (&*list1_ref, &*list2_ref) {
                (Value::List(items1), Value::List(items2)) => {
                    let mut new_items = items1.clone();
                    new_items.extend(items2.iter().cloned());
                    Ok(new_items)
                }
                _ => Err(Error::from(VMError::TypeMismatch(
                    "concatenate lists".to_string(),
                ))),
            }
        }?;

        self.push(shared(Value::List(new_items)))
    }

    /// Removes and returns the first element of a list
    pub fn split_head(&mut self) -> Result<()> {
        let list = self.pop()?;
        let head = (*list)
            .borrow_mut()
            .get_list_mut()
            .ok_or_else(|| VMError::TypeMismatch("split head of list".to_string()))
            .and_then(|l| {
                if l.is_empty() {
                    Err(VMError::EmptyList)
                } else {
                    Ok(l.remove(0))
                }
            })?;

        self.push(head)
    }

    /// Creates a new empty string and pushes it onto the stack
    pub fn create_string(&mut self) -> Result<()> {
        self.push(shared(Value::String(String::new())))
    }

    /// Converts a value to its string representation
    pub fn to_string(&mut self) -> Result<()> {
        let value = self.pop()?.borrow().clone();
        self.push(shared(Value::String(value.to_string())))
    }

    /// Converts a list of UTF-8 byte values to a string
    pub fn utf8_to_string(&mut self) -> Result<()> {
        let list = self.pop()?;
        let list_ref = list.borrow();

        if let Value::List(items) = &*list_ref {
            let mut bytes = Vec::new();
            for item in items {
                match &*item.borrow() {
                    Value::Integer(n) if *n >= 0 && *n <= 255 => bytes.push(*n as u8),
                    _ => return Err(VMError::TypeMismatch("UTF-8 byte value".to_string()).into()),
                }
            }

            match String::from_utf8(bytes) {
                Ok(s) => self.push(shared(Value::String(s))),
                Err(_) => Err(VMError::TypeMismatch("invalid UTF-8 sequence".to_string()).into()),
            }
        } else {
            Err(VMError::TypeMismatch("list of bytes".to_string()).into())
        }
    }

    /// Concatenates two strings
    pub fn string_concat(&mut self) -> Result<()> {
        let b = self.pop()?;
        let a = self.pop()?;

        let result = {
            let a_ref = a.borrow();
            let b_ref = b.borrow();
            match (&*a_ref, &*b_ref) {
                (Value::String(s1), Value::String(s2)) => {
                    let mut new_string = s1.clone();
                    new_string.push_str(s2);
                    Ok(new_string)
                }
                _ => Err(VMError::TypeMismatch("string concatenation".to_string())),
            }
        }?;

        self.push(shared(Value::String(result)))
    }

    /// Calls a function by its index in the op_table
    pub fn call(&mut self) -> Result<()> {
        let fn_index = self
            .environment
            .as_ref()
            .and_then(|e| e.borrow().get_instruction(self.ip))
            .ok_or(VMError::InvalidAddress(self.ip))?;
        self.ip += 1;

        // Save the current IP on the return stack
        self.push_return(shared(Value::Address(self.ip)))?;

        // Jump to the function's code
        self.ip = fn_index;
        Ok(())
    }

    /// Calls a function from the stack
    pub fn call_stack(&mut self) -> Result<()> {
        let func = self.pop()?;
        let vm = self;

        (*func)
            .borrow()
            .get_function()
            .ok_or_else(|| Error::from(VMError::TypeMismatch("function call".to_string())))
            .and_then(|c| c.call(vm))?;

        Ok(())
    }

    /// Defines a named function
    pub fn function(&mut self) -> Result<()> {
        let lambda = self.pop()?;
        let name = self.pop()?;

        let name_str = match &*name.borrow() {
            Value::String(s) => s.clone(),
            _ => return Err(VMError::TypeMismatch("function name".to_string()).into()),
        };

        let callable = (*lambda)
            .borrow_mut()
            .get_function_mut()
            .ok_or_else(|| Error::from(VMError::TypeMismatch("lambda".to_string())))
            .and_then(|c| {
                c.set_name(&name_str)?;
                Ok(c.clone())
            })?;

        // Add the function to the op_table
        if let Some(env) = self.environment.as_ref() {
            let env = env.clone();
            (*env).borrow_mut().add_to_op_table(shared(callable));
        }

        Ok(())
    }

    /// Returns from a function
    pub fn return_op(&mut self) -> Result<()> {
        if self.return_stack.is_empty() {
            // TODO: not wild about using `VMError::Exit` for flow control here.
            return Err(VMError::Exit.into());
        }

        let return_addr = self.pop_return()?;
        let return_addr = return_addr.borrow();
        match &*return_addr {
            Value::Address(addr) => {
                self.ip = *addr;
                Ok(())
            }
            _ => Err(VMError::TypeMismatch("return address".to_string()).into()),
        }
    }

    /// Jumps to a specific instruction
    pub fn jump(&mut self) -> Result<()> {
        let target = self
            .environment
            .as_ref()
            .and_then(|env| env.borrow().get_instruction(self.ip))
            .ok_or(VMError::InvalidAddress(self.ip))?;
        self.ip = target;
        Ok(())
    }

    /// Jumps to an instruction from the stack
    pub fn jump_stack(&mut self) -> Result<()> {
        let target = self.pop()?;
        let target = target.borrow();
        match &*target {
            Value::Address(addr) => {
                self.ip = *addr;
                Ok(())
            }
            _ => Err(VMError::TypeMismatch("jump address".to_string()).into()),
        }
    }

    fn debug_op(&self) {
        let env_loc = EnvLoc::new(self.environment.clone().unwrap(), self.ip);
        log::debug!("{:?}", env_loc);
    }

    fn debug_stacks(&self) {
        let stack_repr = self
            .stack
            .iter()
            .map(|v| format!("[{}]", v.borrow()))
            .collect::<Vec<_>>()
            .join(" ");
        let rstack_repr = self
            .return_stack
            .iter()
            .map(|v| format!("[{}]", v.borrow()))
            .collect::<Vec<_>>()
            .join(" ");
        log::trace!("DATA  : {}\nRETURN: {}\n", stack_repr, rstack_repr);
    }
}

impl Execute for VM {
    /// Returns an iterator over stack values from bottom to top
    fn stack(&self) -> Vec<Value> {
        self.stack
            .iter()
            .map(|shared| shared.borrow().clone())
            .collect()
    }

    /// Runs the VM, executing all instructions in the instruction stream
    fn run(&mut self, env: Shared<Environment>) -> Result<()> {
        self.environment = Some(env.clone());
        let max_ip = self
            .environment
            .as_ref()
            .map(|e| e.borrow().instructions_len())
            .unwrap_or_default();
        while self.ip < max_ip {
            // Get the next instruction (OpCode)
            let op_code = self
                .environment
                .as_ref()
                .and_then(|e| e.borrow().get_instruction(self.ip))
                .ok_or(Error::VMError(VMError::InvalidInstructionPointer(self.ip)))?;

            if log_enabled!(Level::Debug) {
                self.debug_op();
            }
            if log_enabled!(Level::Trace) {
                self.debug_stacks();
            }

            self.ip += 1;

            // Get the operation from the op_table
            let operation = self
                .environment
                .as_ref()
                .and_then(|e| e.borrow().get_op(op_code))
                .ok_or_else(|| Error::VMError(VMError::InvalidOpCode(self.ip - 1, op_code)))?;

            // Execute the operation
            let operation = operation.borrow();
            match *operation {
                Callable::BuiltIn(f) => match f(self) {
                    Ok(()) => {}
                    Err(Error::VMError(VMError::Exit)) => {
                        return Ok(());
                    }
                    err => {
                        // Reset for the next input
                        self.ip = max_ip;
                        self.return_stack.clear();
                        return err;
                    }
                },
                Callable::Fn(Function {
                    ip: instructions, ..
                }) => self.ip = instructions,
            }
        }

        Ok(())
    }

    fn execute_macro(
        &mut self,
        env: Shared<Environment>,
        _trigger: &TokenType,
        function: &Function,
        tokens: &[Value],
    ) -> Result<Vec<Value>> {
        if log::log_enabled!(Level::Trace) {
            trace_list("VM::execute_macro input", 8, tokens);
        }
        // Convert the tokens seen already to a form we can work on.
        let shared_tokens = tokens.into_iter().cloned().map(shared).collect::<Vec<_>>();
        let value = Value::List(shared_tokens);
        self.stack.push(shared(value));

        // Set the IP to the macro, run it, and reset the IP to where it was.
        let macro_ip = function.ip;
        log::trace!("VM::execute_macro running: ip = {}", macro_ip);
        let ip = self.ip;
        self.ip = macro_ip;
        self.run(env.clone())?;
        log::trace!("VM::execute_macro restoring ip to {}", ip);
        self.ip = ip;

        // Get the token list off the stack and return it to the compiler form.
        if let Some(value) = self.stack.pop() {
            if let Some(list) = value.borrow_mut().get_list_mut() {
                let list = list
                    .into_iter()
                    .map(|item| item.borrow().clone())
                    .collect::<Vec<_>>();
                if log::log_enabled!(Level::Trace) {
                    trace_list("VM::execute_macro output", 8, &list);
                }
                Ok(list)
            } else {
                Err(VMError::StackUnderflow.into())
            }
        } else {
            Err(VMError::StackUnderflow.into())
        }
    }
}

fn trace_list<T: Debug>(header: &str, items: usize, list: &[T]) {
    let len = list.len();
    log::trace!(
        "{} {:?}",
        header,
        if len <= items {
            list
        } else {
            &list[len - items..]
        }
    );
}

#[cfg(test)]
mod tests;
