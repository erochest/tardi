use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Callable, Lambda};
use crate::{Compiler, Scanner};
use log::{log_enabled, Level};
use std::fmt::Debug;

use crate::env::{EnvLoc, Environment};
use crate::error::{Error, Result, VMError};

pub mod ops;
pub use self::ops::OpCode;

use crate::value::{SharedValue, Value, ValueData};

use crate::core::Execute;

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

    /// Pushes the current instruction pointer onto the return stack
    pub fn push_ip(&mut self) -> Result<()> {
        log::trace!("pushing instruction pointer onto return stack {}", self.ip);
        self.push_return(shared(ValueData::Address(self.ip).into()))
    }

    /// Pushes a shared value onto the return stack
    pub fn push_return(&mut self, value: SharedValue) -> Result<()> {
        // TODO: change back to 1024
        if self.return_stack.len() >= 12 {
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
        self.stack.pop().ok_or_else(|| {
            log::warn!("VM::pop VMError::StackUnderflow");
            VMError::StackUnderflow.into()
        })
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
            .push(shared(ValueData::Integer(self.stack_size() as i64).into()));
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
        self.push(shared(ValueData::Boolean(a == b).into()))
    }

    /// Compares if a is less than b
    pub fn less(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(ValueData::Boolean(ordering.is_lt()).into())),
            None => Err(VMError::TypeMismatch("less than comparison".to_string()).into()),
        }
    }

    /// Compares if a is greater than b
    pub fn greater(&mut self) -> Result<()> {
        let b = self.pop()?.borrow().clone();
        let a = self.pop()?.borrow().clone();
        match a.partial_cmp(&b) {
            Some(ordering) => self.push(shared(ValueData::Boolean(ordering.is_gt()).into())),
            None => Err(VMError::TypeMismatch("greater than comparison".to_string()).into()),
        }
    }

    /// Performs logical NOT operation on the top stack item
    pub fn not(&mut self) -> Result<()> {
        let value = self.pop()?.borrow().clone();
        match value.data {
            ValueData::Boolean(b) => self.push(shared(ValueData::Boolean(!b).into())),
            _ => Err(VMError::TypeMismatch("logical NOT".to_string()).into()),
        }
    }

    /// Creates a new empty list and pushes it onto the stack
    pub fn create_list(&mut self) -> Result<()> {
        self.push(shared(ValueData::List(Vec::new()).into()))
    }

    /// Appends a value to the end of a list
    pub fn append(&mut self) -> Result<()> {
        let list = self.pop()?;
        let value = self.pop()?;

        if let Some(list) = (*list).borrow_mut().get_list_mut() {
            list.push(value);
        } else {
            return Err(VMError::TypeMismatch(format!("[ {:?} ] {:?} append", list, value)).into());
        }

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
            let list1 = list1.borrow();
            let list2 = list2.borrow();
            let list1_ref = list1.get_list();
            let list2_ref = list2.get_list();
            match (list1_ref, list2_ref) {
                (Some(items1), Some(items2)) => {
                    let mut new_items = items1.clone();
                    new_items.extend(items2.iter().cloned());
                    Ok(new_items)
                }
                _ => Err(Error::from(VMError::TypeMismatch(
                    "concatenate lists".to_string(),
                ))),
            }
        }?;

        self.push(shared(ValueData::List(new_items).into()))
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
        self.push(shared(ValueData::String(String::new()).into()))
    }

    /// Converts a value to its string representation
    pub fn to_string(&mut self) -> Result<()> {
        let value = self.pop()?.borrow().clone();
        self.push(shared(ValueData::String(value.to_string()).into()))
    }

    /// Converts a list of UTF-8 byte values to a string
    pub fn utf8_to_string(&mut self) -> Result<()> {
        let list = self.pop()?;
        let list = list.borrow();
        let list = list.get_list();

        if let Some(items) = list {
            let mut bytes = Vec::new();
            for item in items {
                if let Some(n) = item.borrow().get_integer() {
                    if n >= 0 && n <= 255 {
                        bytes.push(n as u8);
                        continue;
                    }
                }
                return Err(VMError::TypeMismatch("UTF-8 byte value".to_string()).into());
            }

            match String::from_utf8(bytes) {
                Ok(s) => self.push(shared(ValueData::String(s).into())),
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
            let a = a.borrow();
            let a = a.get_string();
            let b = b.borrow();
            let b = b.get_string();
            match (a, b) {
                (Some(s1), Some(s2)) => {
                    let mut new_string = s1.to_string();
                    new_string.push_str(s2);
                    Ok(new_string)
                }
                _ => Err(VMError::TypeMismatch("string concatenation".to_string())),
            }
        }?;

        self.push(shared(ValueData::String(result).into()))
    }

    /// Calls a function by its index in the op_table
    pub fn call(&mut self, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
        let op_table_index = self
            .environment
            .as_ref()
            .and_then(|e| e.borrow().get_instruction(self.ip))
            .ok_or(VMError::InvalidAddress(self.ip))?;
        self.ip += 1;

        let lambda = self
            .environment
            .as_ref()
            .unwrap()
            .borrow()
            .get_callable(op_table_index)
            .ok_or(VMError::InvalidAddress(op_table_index))?;
        lambda.borrow().call(self, compiler, scanner)?;

        Ok(())
    }

    /// Calls a function from the stack
    pub fn call_stack(&mut self, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
        let func = self.pop()?;
        let vm = self;

        (*func)
            .borrow()
            .get_function()
            .ok_or_else(|| Error::from(VMError::TypeMismatch("function call".to_string())))
            .and_then(|c| c.callable.call(vm, compiler, scanner))?;

        Ok(())
    }

    /// Defines a named function
    pub fn function(&mut self) -> Result<()> {
        let lambda = self.pop()?;
        let name = self.pop()?;

        let name_str = match name.borrow().data {
            ValueData::String(ref s) => s.to_string(),
            ValueData::Word(ref w) => w.to_string(),
            _ => return Err(VMError::TypeMismatch("function name".to_string()).into()),
        };

        let callable = (*lambda)
            .borrow_mut()
            .get_function_mut()
            .ok_or_else(|| Error::from(VMError::TypeMismatch("lambda".to_string())))
            .and_then(|c| {
                c.name = Some(name_str);
                Ok(c.clone())
            })?;
        log::trace!("function: {}", callable);

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
        let addr = return_addr
            .get_address()
            .ok_or_else(|| VMError::TypeMismatch("return addres".to_string()))?;
        self.ip = addr;

        Ok(())
    }

    /// Return from a macro
    pub fn exit(&self) -> Result<()> {
        Err(VMError::Exit.into())
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
        let addr = target
            .get_address()
            .ok_or_else(|| VMError::TypeMismatch("jump addres".to_string()))?;
        self.ip = addr;
        Ok(())
    }

    /// Takes a list off the stack and compiles it into an anonymous lambda
    pub fn compile(&mut self, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
        let value = self.pop()?;
        let value = unshare_clone(value);
        if let ValueData::List(words) = value.data {
            let words = words
                .into_iter()
                .map(|word| unshare_clone(word))
                .collect::<Vec<_>>();
            let lambda =
                compiler.compile_list(self, self.environment.clone().unwrap(), scanner, &words)?;
            let value = Value::new(ValueData::Function(lambda));
            self.push(shared(value))?;
        }

        Ok(())
    }

    fn debug_op(&self) {
        let env_loc = EnvLoc::new(self.environment.clone().unwrap(), self.ip);
        let debugged = format!("{:?}", env_loc);
        log::debug!("{}", debugged.trim_end());
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
        log::trace!("DATA  : {}\tRETURN: {}", stack_repr, rstack_repr);
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
    fn run(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        scanner: &mut Scanner,
    ) -> Result<()> {
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

            if log_enabled!(Level::Trace) {
                self.debug_stacks();
            }
            if log_enabled!(Level::Debug) {
                self.debug_op();
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
            let result = operation.callable.call(self, compiler, scanner);
            match result {
                Ok(()) => {}
                Err(Error::VMError(VMError::Exit)) => return Ok(()),
                err => {
                    self.ip = max_ip;
                    self.return_stack.clear();
                    return err;
                }
            }
        }

        Ok(())
    }

    fn execute_macro(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        scanner: &mut Scanner,
        _trigger: &ValueData,
        lambda: &Lambda,
        tokens: Shared<Value>,
    ) -> Result<()> {
        if log::log_enabled!(Level::Trace) {
            log::trace!("VM::execute_macro input {}", tokens.borrow());
        }
        // Convert the tokens seen already to a form we can work on.
        self.stack.push(tokens.clone());

        // TODO: is there something here with pushing the return IP but not popping it or popping too much?
        match lambda.call(self, compiler, scanner) {
            Ok(()) => {}
            Err(Error::VMError(VMError::Exit)) => {}
            Err(err) => return Err(err),
        }
        // It's not currently in an execution loop. Builtins are run
        // immediately, but compiled lambdas have to run in an execution loop to
        // move the IP.
        if lambda.is_compiled() {
            // TODO: DRY these up some
            match self.run(env.clone(), compiler, scanner) {
                Ok(()) => {}
                Err(Error::VMError(VMError::Exit)) => {}
                Err(err) => return Err(err),
            }
        }

        // Get the token list off the stack and return it to the compiler form.
        // TODO: I don't think I need to pop both of these here. Just the accumulator is probably enough.
        if log::log_enabled!(Level::Trace) {
            log::trace!("VM::execute_macro cleaning up");
            self.debug_stacks();
        }
        if let Some(top) = self.stack.pop() {
            if top.borrow().is_list() {
                // Hmm. Since tokens _is_ top and a shared structure, `tokens` should
                // just be undated. :skeptical:
                Ok(())
            } else {
                log::warn!("VM::execute_macro: VMError::TypeMismatch: pop accumulator not a list");
                Err(Error::VMError(VMError::TypeMismatch(format!(
                    "Expected accumulator list from macro output: {:?}",
                    top.borrow()
                ))))
            }
        } else {
            log::warn!("VM::execute_macro: VMError::StackUnderflow: pop value to accumulate");
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
