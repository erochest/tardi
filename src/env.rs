use crate::shared::Shared;
use crate::vm::value::{Callable, Function, Value};
use crate::vm::{create_op_table, OpCode};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::result;

// TODO: need documentation for what these are, how they're used, and the methods that operate on these below
// TODO: add something to store macros
#[derive(Default)]
pub struct Environment {
    constants: Vec<Value>,
    instructions: Vec<usize>,
    op_table: Vec<Shared<Callable>>,
    op_map: HashMap<String, usize>,
}

pub struct EnvLoc {
    env: Shared<Environment>,
    ip: usize,
}

impl fmt::Debug for EnvLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self
            .env
            .borrow()
            .get_instruction(self.ip)
            .map(OpCode::try_from)
            .unwrap()
            .unwrap();
        self.env.borrow().debug_op(&op, f, self.ip)?;
        Ok(())
    }
}

impl EnvLoc {
    pub fn new(env: Shared<Environment>, ip: usize) -> Self {
        Self { env, ip }
    }
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

    pub fn debug(&self) -> String {
        format!("{:?}", self)
    }

    fn debug_op(
        &self,
        op: &OpCode,
        f: &mut fmt::Formatter<'_>,
        ip: usize,
    ) -> result::Result<usize, fmt::Error> {
        match op {
            OpCode::Lit | OpCode::Call => self.debug_const(op, f, ip),
            OpCode::Dup
            | OpCode::Swap
            | OpCode::Rot
            | OpCode::Drop
            | OpCode::StackSize
            | OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Equal
            | OpCode::Less
            | OpCode::Greater
            | OpCode::Not
            | OpCode::ToR
            | OpCode::RFrom
            | OpCode::RFetch
            | OpCode::CreateList
            | OpCode::Append
            | OpCode::Prepend
            | OpCode::Concat
            | OpCode::SplitHead
            | OpCode::CreateString
            | OpCode::ToString
            | OpCode::Utf8ToString
            | OpCode::StringConcat
            | OpCode::CallStack
            | OpCode::Return
            | OpCode::JumpStack
            | OpCode::Function => self.debug_simple(op, f, ip),
            OpCode::Jump => self.debug_jump(op, f, ip),
        }
    }

    fn debug_const(
        &self,
        op: &OpCode,
        f: &mut fmt::Formatter<'_>,
        ip: usize,
    ) -> result::Result<usize, fmt::Error> {
        let mut ip = ip;

        self.write_ip_number(f, ip)?;
        self.write_op_code(f, op)?;

        ip += 1;
        let index = self.instructions[ip];
        let value = &self.constants[index];
        writeln!(f, " {:0>4}. {: <16}", index, value)?;

        Ok(ip)
    }

    fn debug_simple(
        &self,
        op: &OpCode,
        f: &mut fmt::Formatter<'_>,
        ip: usize,
    ) -> result::Result<usize, fmt::Error> {
        self.write_ip_number(f, ip)?;
        self.write_op_code(f, op)?;
        writeln!(f)?;
        Ok(ip)
    }

    fn debug_jump(
        &self,
        op: &OpCode,
        f: &mut fmt::Formatter<'_>,
        ip: usize,
    ) -> result::Result<usize, fmt::Error> {
        let mut ip = ip;

        self.write_ip_number(f, ip)?;
        self.write_op_code(f, op)?;

        ip += 1;
        let index = self.instructions[ip];
        writeln!(f, " {:0>4}", index)?;

        Ok(ip)
    }

    fn write_ip_number(&self, f: &mut fmt::Formatter<'_>, ip: usize) -> fmt::Result {
        write!(f, "{:0>4}. ", ip)
    }

    fn write_op_code(&self, f: &mut fmt::Formatter<'_>, op_code: &OpCode) -> fmt::Result {
        let debugged = format!("{:?}", op_code);
        write!(f, "{: <16} | ", debugged)
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ip = 0;

        while ip < self.instructions.len() {
            let op = OpCode::try_from(self.instructions[ip]).unwrap();
            ip = self.debug_op(&op, f, ip)? + 1;
        }

        Ok(())
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
