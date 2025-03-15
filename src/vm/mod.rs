use crate::error::VMError;

/// Type alias for VM operation functions
pub type OpFn = fn(&mut VM) -> Result<(), VMError>;

/// Enum representing different types of values that can be stored on the stack
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

/// The Virtual Machine implementation using Indirect Threaded Code (ITC)
pub struct VM {
    /// Function pointer table for operation dispatch
    op_table: Vec<OpFn>,
    
    /// Instruction stream (bytecode) containing indices into the op_table
    instructions: Vec<usize>,
    
    /// Instruction pointer tracking the current position in the instruction stream
    ip: usize,
    
    /// Data stack for operation arguments and results
    stack: Vec<Value>,
}

impl VM {
    /// Creates a new VM instance
    pub fn new() -> Self {
        VM {
            op_table: Vec::new(),
            instructions: Vec::new(),
            ip: 0,
            stack: Vec::new(),
        }
    }

    /// Runs the VM, executing all instructions in the instruction stream
    pub fn run(&mut self) -> Result<(), VMError> {
        while self.ip < self.instructions.len() {
            let op_index = self.instructions[self.ip];
            self.ip += 1;
            
            if let Some(op) = self.op_table.get(op_index) {
                op(self)?;
            } else {
                return Err(VMError::InvalidOpCode(op_index));
            }
        }
        Ok(())
    }

    /// Pushes a value onto the data stack
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pops a value from the data stack
    pub fn pop(&mut self) -> Result<Value, VMError> {
        self.stack.pop().ok_or(VMError::StackUnderflow)
    }

    /// Returns the current size of the data stack
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Adds an operation to the op_table and returns its index
    pub fn add_op(&mut self, op: OpFn) -> usize {
        self.op_table.push(op);
        self.op_table.len() - 1
    }

    /// Adds an instruction (op_table index) to the instruction stream
    pub fn add_instruction(&mut self, op_index: usize) {
        self.instructions.push(op_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_operations() {
        let mut vm = VM::new();
        
        // Test push
        vm.push(Value::Integer(42));
        assert_eq!(vm.stack_size(), 1);
        
        // Test pop
        let value = vm.pop().unwrap();
        assert!(matches!(value, Value::Integer(42)));
        
        // Test stack underflow
        assert!(matches!(vm.pop(), Err(VMError::StackUnderflow)));
    }

    #[test]
    fn test_basic_vm_execution() {
        let mut vm = VM::new();
        
        // Add a test operation that pushes an integer
        let op_index = vm.add_op(|vm: &mut VM| {
            vm.push(Value::Integer(123));
            Ok(())
        });
        
        // Add the operation to the instruction stream
        vm.add_instruction(op_index);
        
        // Run the VM
        vm.run().unwrap();
        
        // Verify the result
        let value = vm.pop().unwrap();
        assert!(matches!(value, Value::Integer(123)));
    }

    #[test]
    fn test_invalid_opcode() {
        let mut vm = VM::new();
        
        // Add an invalid operation index
        vm.add_instruction(999); // This index doesn't exist in op_table
        
        // Run should return an error
        assert!(matches!(vm.run(), Err(VMError::InvalidOpCode(_))));
    }
}
