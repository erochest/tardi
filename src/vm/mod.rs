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

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
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
}

// Define the operations
pub fn lit(vm: &mut VM) -> Result<()> {
    vm.lit()
}

pub fn dup(vm: &mut VM) -> Result<()> {
    vm.dup()
}

pub fn swap(vm: &mut VM) -> Result<()> {
    vm.swap()
}

pub fn rot(vm: &mut VM) -> Result<()> {
    vm.rot()
}

pub fn drop_op(vm: &mut VM) -> Result<()> {
    vm.drop_op()
}

// Helper function to add an operation to the table and map
fn add_op(op_table: &mut Vec<OpFn>, op_map: &mut HashMap<String, usize>, op: OpFn, name: &str) {
    let index = op_table.len();
    op_table.push(op);
    op_map.insert(name.to_string(), index);
}

// Create the default operation table
pub fn create_op_table() -> (Vec<OpFn>, HashMap<String, usize>) {
    let mut op_table = Vec::new();
    let mut op_map = HashMap::new();
    
    add_op(&mut op_table, &mut op_map, lit, "lit");
    add_op(&mut op_table, &mut op_map, dup, "dup");
    add_op(&mut op_table, &mut op_map, swap, "swap");
    add_op(&mut op_table, &mut op_map, rot, "rot");
    add_op(&mut op_table, &mut op_map, drop_op, "drop");
    
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
        
        // Test push and pop
        vm.push(Value::Integer(42)).unwrap();
        assert_eq!(vm.stack_size(), 1);
        let value = vm.pop().unwrap();
        assert!(matches!(value, Value::Integer(42)));
        assert!(matches!(vm.pop(), Err(Error::VMError(VMError::StackUnderflow))));

        // Test dup
        vm.push(Value::Integer(1)).unwrap();
        vm.dup().unwrap();
        assert_eq!(vm.stack_size(), 2);
        assert!(matches!(vm.pop().unwrap(), Value::Integer(1)));
        assert!(matches!(vm.pop().unwrap(), Value::Integer(1)));

        // Test swap
        vm.push(Value::Integer(1)).unwrap();
        vm.push(Value::Integer(2)).unwrap();
        vm.swap().unwrap();
        assert!(matches!(vm.pop().unwrap(), Value::Integer(1)));
        assert!(matches!(vm.pop().unwrap(), Value::Integer(2)));

        // Test rot
        vm.push(Value::Integer(1)).unwrap();
        vm.push(Value::Integer(2)).unwrap();
        vm.push(Value::Integer(3)).unwrap();
        vm.rot().unwrap();
        assert!(matches!(vm.pop().unwrap(), Value::Integer(1)));
        assert!(matches!(vm.pop().unwrap(), Value::Integer(3)));
        assert!(matches!(vm.pop().unwrap(), Value::Integer(2)));

        // Test drop_op
        vm.push(Value::Integer(42)).unwrap();
        vm.drop_op().unwrap();
        assert_eq!(vm.stack_size(), 0);
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
