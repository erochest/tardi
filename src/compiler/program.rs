use crate::vm::value::{Callable, Function, Shared, Value};
use crate::vm::{OpCode, Program as VMProgram};
use std::collections::HashMap;

pub struct Program {
    /// Stack of instruction vectors for compiling functions/lambdas
    function_stack: Vec<Vec<usize>>,
    constants: Vec<Value>,
    instructions: Vec<usize>,
    op_table: Vec<Shared<Callable>>,
    op_map: HashMap<String, usize>,
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
    pub fn new() -> Self {
        Program {
            constants: Vec::new(),
            instructions: Vec::new(),
            op_table: Vec::new(),
            op_map: HashMap::new(),
            function_stack: Vec::new(),
        }
    }

    /// Starts a new function definition by pushing a new Vec<usize> onto the function_stack
    pub fn start_function(&mut self) -> usize {
        self.function_stack.push(Vec::new());
        self.function_stack.len() - 1
    }

    /// Ends a function definition by popping the top Vec<usize> from function_stack,
    /// appending it to the main instructions, and returning the start index
    pub fn end_function(&mut self) -> usize {
        if let Some(function_instructions) = self.function_stack.pop() {
            self.extend_instructions(function_instructions)
        } else {
            // If there's no function being defined, return current instruction pointer
            self.instructions.len()
        }
    }

    /// Adds a jump over the new instructions, appends the instructions to the main
    /// instruction vector, and returns the start index
    pub fn extend_instructions(&mut self, mut instructions: Vec<usize>) -> usize {
        // Add a jump instruction to skip over the function code
        let jump_target = self.instructions.len() + 2 + instructions.len();
        self.add_op_arg(OpCode::Jump, jump_target);

        // Store the start position of the function
        let function_start = self.instructions.len();

        // Add the function instructions
        self.instructions.append(&mut instructions);

        // Return the start position
        function_start
    }

    /// Adds an instruction to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn add_instruction(&mut self, op_index: usize) {
        if let Some(current_function) = self.function_stack.last_mut() {
            current_function.push(op_index);
        } else {
            self.instructions.push(op_index);
        }
    }

    /// Adds an opcode to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn add_op(&mut self, op: OpCode) {
        self.add_instruction(op.into());
    }

    /// Adds an opcode and its argument to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn add_op_arg(&mut self, op: OpCode, arg: usize) {
        self.add_op(op);
        self.add_instruction(arg);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_constant(&self, index: usize) -> Option<&Value> {
        self.constants.get(index)
    }

    pub fn set_op_table(&mut self, op_table: Vec<Shared<Callable>>) {
        self.op_table = op_table;
    }

    pub fn set_op_map(&mut self, op_map: HashMap<String, usize>) {
        self.op_map = op_map;
    }

    pub fn get_op_table_size(&self) -> usize {
        self.op_table.len()
    }

    pub fn get_instructions(&self) -> &Vec<usize> {
        &self.instructions
    }

    pub fn get_op_name(&self, op_code: usize) -> Option<String> {
        self.op_map
            .iter()
            .find(|(_, &index)| index == op_code)
            .map(|(name, _)| name.clone())
    }

    pub fn get_op_map(&self) -> &HashMap<String, usize> {
        &self.op_map
    }
}

impl VMProgram for Program {
    fn get_instruction(&self, ip: usize) -> Option<usize> {
        self.instructions.get(ip).copied()
    }

    fn get_constant(&self, index: usize) -> Option<&Value> {
        self.constants.get(index)
    }

    fn get_op(&self, index: usize) -> Option<Shared<Callable>> {
        self.op_table.get(index).cloned()
    }

    fn instructions_len(&self) -> usize {
        self.instructions.len()
    }

    fn get_op_table_size(&self) -> usize {
        self.op_table.len()
    }

    fn get_op_map(&self) -> &std::collections::HashMap<String, usize> {
        &self.op_map
    }

    fn add_to_op_table(&mut self, callable: Shared<Callable>) -> usize {
        let index = self.op_table.len();

        if let Callable::Fn(Function {
            name: Some(ref n), ..
        }) = *callable.borrow()
        {
            self.op_map.insert(n.clone(), index);
        }
        self.op_table.push(callable);

        index
    }
}

// We can't derive Clone for Program because OpFn (function pointers) don't implement Clone
// Instead, we implement Clone manually, copying the function pointers directly
impl Clone for Program {
    fn clone(&self) -> Self {
        Program {
            function_stack: vec![],
            constants: self.constants.clone(),
            instructions: self.instructions.clone(),
            op_table: self.op_table.clone(),
            op_map: self.op_map.clone(),
        }
    }
}
