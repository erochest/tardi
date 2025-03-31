use crate::vm::create_op_table;
use crate::vm::value::{Callable, Function, Shared, Value};
use std::collections::HashMap;

#[derive(Default)]
pub struct Environment {
    constants: Vec<Value>,
    instructions: Vec<usize>,
    op_table: Vec<Shared<Callable>>,
    op_map: HashMap<String, usize>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            constants: Vec::new(),
            instructions: Vec::new(),
            op_table: Vec::new(),
            op_map: HashMap::new(),
        }
    }

    pub fn from_parameters(
        constants: Vec<Value>,
        instructions: Vec<usize>,
        op_table: Vec<Shared<Callable>>,
        op_map: HashMap<String, usize>,
    ) -> Self {
        Environment {
            constants,
            instructions,
            op_table,
            op_map,
        }
    }

    pub fn with_builtins() -> Self {
        let mut env = Self::new();
        let op_table = create_op_table();
        env.set_op_table(op_table);
        env
    }

    /// Appends the instructions to the main instruction vector, and returns the
    /// start index.
    pub fn extend_instructions(&mut self, mut instructions: Vec<usize>) -> usize {
        let function_start = self.instructions.len();
        self.instructions.append(&mut instructions);
        function_start
    }

    /// Adds an instruction to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn add_instruction(&mut self, op_code: usize) {
        self.instructions.push(op_code);
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

    pub fn get_instruction(&self, ip: usize) -> Option<usize> {
        self.instructions.get(ip).copied()
    }

    pub fn get_op(&self, index: usize) -> Option<Shared<Callable>> {
        self.op_table.get(index).cloned()
    }

    pub fn instructions_len(&self) -> usize {
        self.instructions.len()
    }

    pub fn add_to_op_table(&mut self, callable: Shared<Callable>) -> usize {
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

// We can't derive Clone for env because OpFn (function pointers) don't implement Clone
// Instead, we implement Clone manually, copying the function pointers directly
impl Clone for Environment {
    fn clone(&self) -> Self {
        Environment {
            constants: self.constants.clone(),
            instructions: self.instructions.clone(),
            op_table: self.op_table.clone(),
            op_map: self.op_map.clone(),
        }
    }
}
