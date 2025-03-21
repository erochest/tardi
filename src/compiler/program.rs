use crate::vm::{OpFn, Program as VMProgram};
use crate::vm::value::Value;
use std::collections::HashMap;

pub struct Program {
    constants: Vec<Value>,
    instructions: Vec<usize>,
    op_table: Vec<OpFn>,
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
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_constant(&self, index: usize) -> Option<&Value> {
        self.constants.get(index)
    }

    pub fn add_instruction(&mut self, op_index: usize) {
        self.instructions.push(op_index);
    }

    pub fn set_op_table(&mut self, op_table: Vec<OpFn>, op_map: HashMap<String, usize>) {
        self.op_table = op_table;
        self.op_map = op_map;
    }

    pub fn get_op_index(&self, op_name: &str) -> Option<usize> {
        self.op_map.get(op_name).copied()
    }

    pub fn get_op_name(&self, op_code: usize) -> Option<String> {
        self.op_map
            .iter()
            .filter(|(_, v)| **v == op_code)
            .map(|(k, _)| k.to_string())
            .next()
    }

    pub fn get_op_table_size(&self) -> usize {
        self.op_table.len()
    }

    pub fn get_instructions(&self) -> &Vec<usize> {
        &self.instructions
    }
}

impl VMProgram for Program {
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

// We can't derive Clone for Program because OpFn (function pointers) don't implement Clone
// Instead, we implement Clone manually, copying the function pointers directly
impl Clone for Program {
    fn clone(&self) -> Self {
        Program {
            constants: self.constants.clone(),
            instructions: self.instructions.clone(),
            op_table: self.op_table.clone(),
            op_map: self.op_map.clone(),
        }
    }
}
