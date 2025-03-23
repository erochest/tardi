use crate::vm::value::Value;
use crate::vm::{OpCode, OpFn, Program as VMProgram};
use std::collections::HashMap;

// TODO:
// Most of this should be handled in the Compiler, not the Program,
// - Create a stack of instruction vectors
// - As new functions/lambdas are started, start a new instruction vector on
//   the stack and for functions, pre-allocate them on the op_map and op_table
// - As functions/lambdas are completed, pop them off the definition stack
//   and append them to the global instructions vector. The method on
//   Program that does this will return the pointer to the start of the
//   function in the global vector
// - the Compiler can use this to complete the function
// - lambdas create an Address constant that gets loaded using `lit`
//
// Method to add:
// - extend_instructions

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

    pub fn add_op(&mut self, op: OpCode) {
        self.instructions.push(op.into());
    }

    pub fn add_op_arg(&mut self, op: OpCode, arg: usize) {
        self.instructions.push(op.into());
        self.instructions.push(arg);
    }

    pub fn set_op_table(&mut self, op_table: Vec<OpFn>) {
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
