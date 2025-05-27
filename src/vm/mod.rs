use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::Lambda;
use crate::Compiler;
use log::{log_enabled, Level};

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

    // TODO: make this smaller and more optimized?
    // TODO: there are also limits that usize places on things that i can't
    // enforce well because i don't know exactly what it is. maybe use u64 or
    // something definite?
    /// Instruction pointer tracking the current position in the instruction stream
    pub ip: usize,

    /// Data stack for operation arguments and results
    pub stack: Vec<SharedValue>,

    /// Return stack for control flow
    pub return_stack: Vec<SharedValue>,

    /// A stack of the module we're currently executing.
    pub module_stack: Vec<String>,
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
            module_stack: Vec::new(),
        }
    }

    /// Pushes the current instruction pointer onto the return stack
    pub fn push_ip(&mut self, is_breakpoint: bool) -> Result<()> {
        log::trace!("pushing instruction pointer onto return stack {}", self.ip);
        self.push_return(shared(ValueData::Return(self.ip, is_breakpoint).into()))
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
        self.stack.pop().ok_or_else(|| {
            log::warn!("VM::pop VMError::StackUnderflow");
            VMError::StackUnderflow.into()
        })
    }

    /// Peek at the top of the stack
    pub fn peek(&self) -> Option<&SharedValue> {
        self.stack.last()
    }

    /// Returns the current size of the data stack
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Executes the lit operation - loads a constant onto the stack
    pub fn lit(&mut self) -> Result<()> {
        let const_index = self
            .environment
            .as_ref()
            .ok_or(VMError::MissingEnvironment)
            .and_then(|e| e.borrow().get_instruction(self.ip))?;
        self.ip += 1;

        let value = self
            .environment
            .as_ref()
            .ok_or(VMError::MissingEnvironment)
            .and_then(|e| {
                e.borrow()
                    .get_constant(const_index)
                    .cloned()
                    .ok_or(VMError::InvalidConstantIndex(const_index))
            })
            .map(shared)?;
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

    /// Clears the stack
    pub fn clear(&mut self) -> Result<()> {
        self.stack.clear();
        Ok(())
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

    /// Evaluates a value and leaves one of two others on the stack.
    ///     condition if-true if-false ?
    /// So
    ///     `#t 1 2 ?` evaluates to `1`
    ///     `#f 1 2 ?` evaluates to `2`
    pub fn question(&mut self) -> Result<()> {
        let if_false = self.pop()?;
        let if_true = self.pop()?;
        let condition = self.pop()?;

        log::trace!("VM::question condition {}", condition.borrow());
        if let Some(b) = condition.borrow().get_boolean() {
            if b {
                self.push(if_true.clone())?;
            } else {
                self.push(if_false.clone())?;
            }
        } else {
            return Err(
                VMError::TypeMismatch("? conditional must be a boolean".to_string()).into(),
            );
        }

        Ok(())
    }

    /// Calls a function by its index in the op_table
    pub fn call(&mut self, compiler: &mut Compiler) -> Result<()> {
        // TODO: probably need to be more defensive about this.
        let env = self
            .environment
            .as_ref()
            .ok_or(VMError::MissingEnvironment)?;
        let op_table_index = env.borrow().get_instruction(self.ip)?;
        self.ip += 1;

        let lambda = env
            .borrow()
            .get_callable(op_table_index)
            .ok_or(VMError::InvalidAddress(op_table_index))?;
        lambda.borrow().call(self, compiler)?;

        Ok(())
    }

    /// Calls a function from the stack
    pub fn apply(&mut self, compiler: &mut Compiler) -> Result<()> {
        let func = self.pop()?;
        let vm = self;

        (*func)
            .borrow()
            .get_function()
            .ok_or_else(|| {
                Error::from(VMError::TypeMismatch(format!(
                    "not a word: {}",
                    func.borrow()
                )))
            })
            .and_then(|c| c.call(vm, compiler))?;

        Ok(())
    }

    /// Defines a named function
    pub fn function(&mut self) -> Result<()> {
        let lambda = self.pop()?;
        let name = self.pop()?;

        let name = name.borrow();
        let (module_name, name_str) = name
            .get_symbol()
            .ok_or_else(|| VMError::TypeMismatch(format!("function name: {:?}", name)))?;
        log::trace!("VM::function {}::{}", module_name, name_str);
        if log::log_enabled!(Level::Trace) {
            let module_names = self.module_stack.join(" :: ");
            log::trace!("module stack '{}'", module_names);
        }

        let env = self
            .environment
            .as_ref()
            .ok_or(VMError::MissingEnvironment)?;

        // Define a predeclared word
        let get_op_index = env.borrow().get_op_index(module_name, name_str);
        if let Some(index) = get_op_index {
            log::trace!("VM::function defining predeclared function {}", name_str);
            let (ip, length) = (*lambda)
                .borrow()
                .get_function()
                .and_then(|f| {
                    let ip = f.get_ip()?;
                    let length = f.get_length()?;
                    Some((ip, length))
                })
                .unwrap(); // TODO: be more defensive here
            let function = &env.borrow_mut().get_op(&ip, index)?;
            (*function).borrow_mut().define_function(ip, length)?;
            return Ok(());
        }

        // Define a word
        let callable = (*lambda)
            .borrow_mut()
            .get_function_mut()
            .ok_or_else(|| Error::from(VMError::TypeMismatch("lambda".to_string())))
            .map(|c| {
                c.name = Some(name_str.to_string());
                c.clone()
            })?;
        log::trace!("function: {}", callable);

        env.borrow_mut()
            .add_to_op_table(module_name, shared(callable))?;

        Ok(())
    }

    pub fn predeclare_function(&mut self) -> Result<()> {
        let name = self.pop()?;

        let name = name.borrow();
        let (module_name, name_str) = name
            .get_symbol()
            .ok_or_else(|| VMError::TypeMismatch("function name".to_string()))?;
        log::trace!("VM::predefine_function {}::{}", module_name, name_str);

        let lambda = Lambda::new_undefined(name_str);
        // Add the function to the op_table
        self.environment
            .as_ref()
            .ok_or(VMError::MissingEnvironment)?
            .borrow_mut()
            .add_to_op_table(module_name, shared(lambda))?;

        Ok(())
    }

    /// Returns from a function
    pub fn return_op(&mut self) -> Result<()> {
        log::trace!("VM::return_op");
        if self.return_stack.is_empty() {
            // TODO: not wild about using `VMError::Stop` for flow control here.
            return Err(VMError::Stop.into());
        }

        let return_addr = self.pop_return()?;
        let return_addr = return_addr.borrow();
        let addr = return_addr
            .get_address()
            .ok_or_else(|| VMError::TypeMismatch("return addres".to_string()))?;
        self.ip = addr;

        Ok(())
    }

    /// Break from a loop by removing things from the return stack until getting to a breakpoint
    /// and then jumping to the next ip value.
    pub fn clear_jump(&mut self) -> Result<()> {
        log::trace!("VM::clear_jump");
        if self.return_stack.is_empty() {
            // TODO: not wild about using `VMError::Stop` for flow control here.
            return Err(VMError::Stop.into());
        }

        while let Some(address) = self.return_stack.last() {
            if address.borrow().is_breakpoint() {
                break;
            }
            self.pop_return()?;
        }

        self.jump()
    }

    /// Return from a macro
    pub fn stop(&self) -> Result<()> {
        // I tried calling `return_op` here, but it seemed to break
        // things in weird ways. Maybe I'll try again and run it
        // down someday.
        log::trace!("stop");
        Err(VMError::Stop.into())
    }

    /// Exit the VM
    pub fn bye(&self) -> Result<()> {
        log::trace!("bye");
        Err(VMError::Bye.into())
    }

    /// Jumps to a specific instruction
    pub fn jump(&mut self) -> Result<()> {
        let target = self
            .environment
            .as_ref()
            .ok_or(VMError::MissingEnvironment)
            .and_then(|env| env.borrow().get_instruction(self.ip))?;
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
    pub fn compile(&mut self, compiler: &mut Compiler) -> Result<()> {
        log::trace!("VM::compile");
        let value = self.pop()?;
        let value = unshare_clone(value);
        if let ValueData::List(words) = value.data {
            let words = words.into_iter().map(unshare_clone).collect::<Vec<_>>();
            let lambda = compiler.compile_list(self, &words)?;
            let value = Value::new(ValueData::Function(lambda));
            self.push(shared(value))?;
        }

        Ok(())
    }

    fn debug_op(&self) {
        let env_loc = EnvLoc::new(self.environment.clone().unwrap(), self.ip);
        let debugged = format!("{:?}", env_loc);
        log::debug!("IP: {}", debugged.trim_end());
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
        log::debug!("DATA  : {}\tRETURN: {}", stack_repr, rstack_repr);
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
    fn run(&mut self, env: Shared<Environment>, compiler: &mut Compiler) -> Result<()> {
        self.environment = Some(env.clone());
        let max_ip = self
            .environment
            .as_ref()
            .unwrap()
            .borrow()
            .instructions_len();
        while self.ip < max_ip {
            // Get the next instruction (OpCode)
            let op_code = self
                .environment
                .as_ref()
                .ok_or(VMError::MissingEnvironment)
                .and_then(|e| e.borrow().get_instruction(self.ip))?;

            if log_enabled!(Level::Trace) {
                self.debug_stacks();
            }
            if log_enabled!(Level::Debug) {
                self.debug_stacks();
                self.debug_op();
            }

            // Get the operation from the op_table
            let operation = self
                .environment
                .as_ref()
                .ok_or(VMError::MissingEnvironment)
                .and_then(|e| e.borrow().get_op(&self.ip, op_code))?;

            self.ip += 1;

            // Execute the operation
            let operation = operation.borrow();
            let result = operation.call(self, compiler);
            match result {
                Ok(()) => {}
                Err(Error::VMError(VMError::Stop)) => {
                    log::trace!("stopping");
                    return Ok(());
                }
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
        trigger: &ValueData,
        lambda: &Lambda,
        tokens: Shared<Value>,
    ) -> Result<()> {
        log::trace!(
            "VM::execute_macro {} -- current input {}",
            trigger,
            tokens.borrow()
        );
        // Convert the tokens seen already to a form we can work on.
        self.stack.push(tokens.clone());

        match lambda.call(self, compiler) {
            Ok(()) => {
                log::trace!(
                    "VM::execute_macro {}: received success from macro call. continuing.",
                    trigger
                )
            }
            Err(Error::VMError(VMError::Stop)) => {
                log::trace!(
                    "VM::execute_macro {}: received stop from macro call. continuing.",
                    trigger
                )
            }
            Err(err) => {
                log::trace!(
                    "VM::execute_macro {}: received error from macro call. bailing.",
                    trigger
                );
                return Err(err);
            }
        }
        // It's not currently in an execution loop. Builtins are run
        // immediately, but compiled lambdas have to run in an execution loop to
        // move the IP.
        if lambda.is_compiled() {
            // TODO: DRY these up some
            log::trace!(
                "VM::execute_macro: {:?} is compiled. executing ip",
                lambda.name
            );
            match self.run(env.clone(), compiler) {
                Ok(()) => {
                    self.return_op()?;
                    log::trace!(
                        "VM::execute_macro {}: received success from macro run. continuing.",
                        trigger
                    )
                }
                Err(Error::VMError(VMError::Stop)) => {
                    self.return_op()?;
                    log::trace!(
                        "VM::execute_macro {}: received stop from macro run. continuing.",
                        trigger
                    )
                }
                Err(err) => {
                    log::trace!(
                        "VM::execute_macro {}: received error from macro run. bailing.",
                        trigger
                    );
                    return Err(err);
                }
            }
        }

        // Get the token list off the stack and return it to the compiler form.
        if log::log_enabled!(Level::Trace) {
            log::trace!("VM::execute_macro {} cleaning up", trigger);
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

#[cfg(test)]
mod tests;
