use crate::core::{create_macro_table, create_op_table};
use crate::error::{Error, Result};
use crate::image::ImageFormat;
use crate::shared::{shared, Shared};
use crate::value::lambda::{Lambda, OpFn};
use crate::value::{Value, ValueData};
use crate::vm::OpCode;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::result;

// TODO: need documentation for what these are, how they're used, and the methods that operate on these below
#[derive(Default)]
pub struct Environment {
    pub constants: Vec<Value>,
    pub instructions: Vec<usize>,
    pub op_table: Vec<Shared<Lambda>>,
    pub op_map: HashMap<String, usize>,
    pub macro_table: HashMap<String, Lambda>,
}

pub struct EnvLoc {
    env: Shared<Environment>,
    ip: usize,
}

impl fmt::Debug for EnvLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.env.borrow().debug_instruction(f, self.ip)?;
        Ok(())
    }
}

impl EnvLoc {
    pub fn new(env: Shared<Environment>, ip: usize) -> Self {
        Self { env, ip }
    }
}

impl TryFrom<ImageFormat> for Environment {
    type Error = Error;

    fn try_from(image: ImageFormat) -> result::Result<Self, Self::Error> {
        let op_table = create_op_table();
        let builtins: HashMap<String, OpFn> = op_table
            .iter()
            .filter_map(|lambda| {
                let lambda = lambda.borrow();
                match (lambda.name.as_ref(), lambda.get_builtin_fn()) {
                    (Some(name), Some(function)) => Some((name.clone(), function.clone())),
                    _ => None,
                }
            })
            .collect();

        let env = &image.env;
        let op_table: Vec<Shared<Lambda>> = env
            .op_table
            .into_iter()
            .filter_map(|sl| sl.into_lambda(&builtins).ok())
            .map(shared)
            .collect();

        let macro_table: HashMap<String, Lambda> = env
            .macro_table
            .into_iter()
            .map(|(k, sl)| Ok((k, sl.into_lambda(&builtins)?)))
            .collect::<Result<_>>()?;

        Ok(Environment::from_parameters(
            env.constants,
            env.instructions,
            op_table,
            env.op_map,
            macro_table,
        ))
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            constants: Vec::new(),
            instructions: Vec::new(),
            op_table: Vec::new(),
            op_map: HashMap::new(),
            macro_table: HashMap::new(),
        }
    }

    pub fn from_parameters(
        constants: Vec<Value>,
        instructions: Vec<usize>,
        op_table: Vec<Shared<Lambda>>,
        op_map: HashMap<String, usize>,
        macro_table: HashMap<String, Lambda>,
    ) -> Self {
        Environment {
            constants,
            instructions,
            op_table,
            op_map,
            macro_table,
        }
    }

    pub fn with_builtins() -> Self {
        let mut env = Self::new();
        let op_table = create_op_table();
        env.set_op_table(op_table);
        let macro_table = create_macro_table();
        env.set_macro_table(macro_table);
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

    pub fn set_op_table(&mut self, op_table: Vec<Shared<Lambda>>) {
        self.op_table = op_table;
    }

    pub fn set_macro_table(&mut self, macro_table: HashMap<String, Lambda>) {
        self.macro_table = macro_table;
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

    pub fn get_op(&self, index: usize) -> Option<Shared<Lambda>> {
        self.op_table.get(index).cloned()
    }

    pub fn instructions_len(&self) -> usize {
        self.instructions.len()
    }

    /// Add a new function to the op_table. If the function has a name, add it to the op_map.
    /// The value in `op_map` is the index of the function in `op_table`.
    /// The value in `op_table` is the function itself.
    pub fn add_to_op_table(&mut self, lambda: Shared<Lambda>) -> usize {
        let index = self.op_table.len();

        if let Some(ref n) = lambda.borrow().name {
            self.op_map.insert(n.to_string(), index);
        }
        self.op_table.push(lambda);

        index
    }

    pub fn get_callable(&self, index: usize) -> Option<Shared<Lambda>> {
        self.op_table.get(index).cloned()
    }

    pub fn add_macro(&mut self, lambda: Lambda) -> Result<()> {
        let key = lambda.name.clone().unwrap_or_default();
        log::trace!("Environment::add_macro {:?}", key);
        self.macro_table.insert(key.to_string(), lambda);
        Ok(())
    }

    pub fn is_macro_trigger(&self, trigger: &ValueData) -> bool {
        self.macro_table.contains_key(&trigger.to_string())
    }

    pub fn get_macro(&self, trigger: &ValueData) -> Option<&Lambda> {
        self.macro_table.get(&trigger.to_string())
    }

    pub fn debug(&self) -> String {
        format!("{:?}", self)
    }

    fn debug_instruction(
        &self,
        f: &mut fmt::Formatter<'_>,
        ip: usize,
    ) -> result::Result<usize, fmt::Error> {
        let mut ip = ip;
        let instruction = self.instructions[ip];
        ip = match OpCode::try_from(instruction) {
            Ok(op) => self.debug_op(&op, f, ip)?,
            Err(_) => self.debug_call(instruction, f, ip)?,
        };
        writeln!(f)?;
        Ok(ip + 1)
    }

    fn debug_op(
        &self,
        op: &OpCode,
        f: &mut fmt::Formatter<'_>,
        ip: usize,
    ) -> result::Result<usize, fmt::Error> {
        let next_ip = match op {
            OpCode::Lit => self.debug_const(op, f, ip),
            OpCode::Dup
            | OpCode::Swap
            | OpCode::Rot
            | OpCode::Drop
            | OpCode::Clear
            | OpCode::StackSize
            | OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Equal
            | OpCode::Less
            | OpCode::Greater
            | OpCode::Not
            | OpCode::Question
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
            | OpCode::Apply
            | OpCode::Return
            | OpCode::Exit
            | OpCode::JumpStack
            | OpCode::Function
            | OpCode::PredeclareFunction
            | OpCode::ScanValue
            | OpCode::ScanValueList
            | OpCode::ScanObjectList
            | OpCode::LitStack
            | OpCode::Compile
            | OpCode::Dump => self.debug_simple(op, f, ip),
            OpCode::Jump => self.debug_jump(op, f, ip),
        }?;

        self.write_function_names(f, ip)?;

        Ok(next_ip)
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
        write!(f, " {:0>4}. {: <16}", index, value)?;

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
        write!(f, " {:0>4}", index)?;

        Ok(ip)
    }

    fn debug_call(
        &self,
        index: usize,
        f: &mut fmt::Formatter<'_>,
        ip: usize,
    ) -> result::Result<usize, fmt::Error> {
        let op = self.op_table[index].clone();
        let name = op
            .borrow()
            .name
            .clone()
            .unwrap_or_else(|| "<lambda>".to_string());

        self.write_ip_number(f, ip)?;
        self.write_call(f, index, &name)?;

        // should be safe to unwrap because these should only be compiled
        write!(f, " {:0>4}", op.borrow().get_ip().unwrap())?;

        Ok(ip)
    }

    fn write_ip_number(&self, f: &mut fmt::Formatter<'_>, ip: usize) -> fmt::Result {
        write!(f, "{:0>4}. ", ip)
    }

    fn write_op_code(&self, f: &mut fmt::Formatter<'_>, op_code: &OpCode) -> fmt::Result {
        let debugged = format!("{:?}", op_code);
        write!(f, "{: <16} | ", debugged)
    }

    fn write_call(&self, f: &mut fmt::Formatter<'_>, index: usize, name: &str) -> fmt::Result {
        write!(f, "{:0>4}. {: <10} | ", index, name)
    }

    fn write_function_names(&self, f: &mut fmt::Formatter<'_>, ip: usize) -> fmt::Result {
        let mut names = self
            .op_map
            .iter()
            .filter(|(_n, index)| {
                self.op_table.get(**index).is_some_and(|lambda| {
                    lambda
                        .borrow()
                        .get_ip()
                        .is_some_and(|lambda_ip| lambda_ip == ip)
                })
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        names.extend(
            self.macro_table
                .iter()
                .filter(|(_n, lambda)| lambda.get_ip().is_some_and(|lambda_ip| lambda_ip == ip))
                .map(|(name, _)| name.clone()),
        );
        let names = names.join(" ");

        // TODO: sometimes the column before this is omitted. Make them line up.
        if !names.is_empty() {
            write!(f, " {: <16} | ", names)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ip = 0;

        while ip < self.instructions.len() {
            ip = self.debug_instruction(f, ip)?;
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
            macro_table: self.macro_table.clone(),
        }
    }
}
