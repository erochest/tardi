use crate::error::{Result, Error, VMError};
use std::collections::HashMap;
use std::fmt;

/// Function pointer type for VM operations
pub type OpFn = fn(&mut VM) -> Result<()>;

/// Trait for programs that can be executed by the VM
pub trait Program: 'static {
    fn get_instruction(&self, ip: usize) -> Option<usize>;
    fn get_constant(&self, index: usize) -> Option<&Value>;
    fn get_op(&self, index: usize) -> Option<&OpFn>;
    fn instructions_len(&self) -> usize;
}

/// Enum representing different types of values that can be stored on the stack
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(x) => write!(f, "{}", x),
            Value::Boolean(true) => write!(f, "#t"),
            Value::Boolean(false) => write!(f, "#f"),
        }
    }
}

/// The Virtual Machine implementation using Indirect Threaded Code (ITC)
pub struct VM {
    /// The program being executed
    program: Option<Box<dyn Program>>,
    
    /// Instruction pointer tracking the current position in the instruction stream
    ip: usize,
    
    /// Data stack for operation arguments and results
    stack: Vec<Value>,
}

impl VM {
    /// Returns an iterator over stack values from bottom to top
    pub fn stack_iter(&self) -> impl Iterator<Item = &Value> {
        self.stack.iter()
    }

    /// Creates a new VM instance
    pub fn new() -> Self {
        VM {
            program: None,
            ip: 0,
            stack: Vec::new(),
        }
    }

    /// Loads a program into the VM
    pub fn load_program(&mut self, program: Box<dyn Program>) {
        self.program = Some(program);
        self.ip = 0;
    }

    /// Runs the VM, executing all instructions in the instruction stream
    pub fn run(&mut self) -> Result<()> {
        while let Some(program) = &self.program {
            if self.ip >= program.instructions_len() {
                break;
            }

            // Get the next instruction and operation
            let op_index = program.get_instruction(self.ip)
                .ok_or(Error::VMError(VMError::InvalidOpCode(self.ip)))?;
            let operation = program.get_op(op_index)
                .ok_or(Error::VMError(VMError::InvalidOpCode(op_index)))?;
            
            // Store the operation in a local variable
            let op = *operation;
            self.ip += 1;

            // Execute the operation
            op(self)?;
        }

        Ok(())
    }

    /// Pushes a value onto the data stack
    pub fn push(&mut self, value: Value) -> Result<()> {
        if self.stack.len() >= 1024 {
            return Err(VMError::StackOverflow.into());
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pops a value from the data stack
    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(VMError::StackUnderflow.into())
    }

    /// Returns the current size of the data stack
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Executes the lit operation - loads a constant onto the stack
    pub fn lit(&mut self) -> Result<()> {
        let program = self.program.as_ref().ok_or(Error::VMError(VMError::NoProgram))?;
        
        let const_index = program.get_instruction(self.ip)
            .ok_or(Error::VMError(VMError::InvalidOpCode(self.ip)))?;
        self.ip += 1;
        
        if let Some(value) = program.get_constant(const_index) {
            self.push(value.clone())
        } else {
            Err(Error::VMError(VMError::InvalidConstantIndex(const_index)))
        }
    }
}

// Define the operations
pub fn lit(vm: &mut VM) -> Result<()> {
    vm.lit()
}

// Create the default operation table
pub fn create_op_table() -> (Vec<OpFn>, HashMap<String, usize>) {
    let mut op_table = Vec::new();
    let mut op_map = HashMap::new();
    
    op_table.push(lit as OpFn);
    op_map.insert("lit".to_string(), 0);
    
    (op_table, op_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProgram {
        instructions: Vec<usize>,
        constants: Vec<Value>,
        op_table: Vec<OpFn>,
    }

    impl Program for TestProgram {
        fn get_instruction(&self, ip: usize) -> Option<usize> {
            self.instructions.get(ip).copied()
        }

        fn get_constant(&self, index: usize) -> Option<&Value> {
            self.constants.get(index)
        }

        fn get_op(&self, index: usize) -> Option<&OpFn> {
            self.op_table.get(index)
        }

        fn instructions_len(&self) -> usize {
            self.instructions.len()
        }
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = VM::new();
        
        // Test push
        vm.push(Value::Integer(42)).unwrap();
        assert_eq!(vm.stack_size(), 1);
        
        // Test pop
        let value = vm.pop().unwrap();
        assert!(matches!(value, Value::Integer(42)));
        
        // Test stack underflow
        assert!(matches!(vm.pop(), Err(Error::VMError(VMError::StackUnderflow))));
    }

    #[test]
    fn test_basic_vm_execution() {
        let mut vm = VM::new();
        let (op_table, _) = create_op_table();
        
        let program = Box::new(TestProgram {
            instructions: vec![0, 0], // lit operation index followed by constant index
            constants: vec![Value::Integer(123)],
            op_table,
        });
        
        vm.load_program(program);
        vm.run().unwrap();
        
        // Verify the result
        let value = vm.pop().unwrap();
        assert!(matches!(value, Value::Integer(123)));
    }

    #[test]
    fn test_invalid_opcode() {
        let mut vm = VM::new();
        let program = Box::new(TestProgram {
            instructions: vec![999], // Invalid opcode
            constants: vec![],
            op_table: vec![],
        });
        
        vm.load_program(program);
        assert!(matches!(vm.run(), Err(Error::VMError(VMError::InvalidOpCode(_)))));
    }
}
