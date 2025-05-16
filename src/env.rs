use crate::compiler::error::{CompilerError, CompilerResult};
use crate::compiler::module::KERNEL;
use crate::core::{create_kernel_module, create_op_table};
use crate::error::{Result, VMError, VMResult};
use crate::shared::{shared, Shared};
use crate::value::lambda::Lambda;
use crate::value::Value;
use crate::vm::OpCode;
use crate::Scanner;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::path::{Path, PathBuf};
use std::result;

// TODO: move this to module/mod.rs
#[derive(Default, Clone)]
pub struct Module {
    pub path: Option<PathBuf>,
    pub name: String,

    /// This maps a word name to its index in the environment's `op_table`.
    pub defined: HashMap<String, usize>,

    /// This maps the imported word names to their indexes in the environment's `op_table`.
    pub imported: HashMap<String, usize>,
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "MODULE  : {} / {}",
            self.name,
            self.path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        )?;
        for (name, index) in self.defined.iter() {
            writeln!(f, "\tDEFINED : {:20} => {}", name, index)?;
        }
        for (name, index) in self.imported.iter() {
            writeln!(f, "\tIMPORTED: {:20} => {}", name, index)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Module {
    pub fn new(name: &str) -> Module {
        let name = name.to_string();
        Module {
            path: None,
            name,
            defined: HashMap::new(),
            imported: HashMap::new(),
        }
    }

    pub fn with_path(path: &Path, name: &str) -> Module {
        let path = Some(path.to_path_buf());
        let name = name.to_string();
        Module {
            path,
            name,
            defined: HashMap::new(),
            imported: HashMap::new(),
        }
    }

    pub fn with_imports(name: &str, module: &Module) -> Module {
        let name = name.to_string();
        let imported = module.defined.clone();
        Module {
            path: None,
            name,
            defined: HashMap::new(),
            imported,
        }
    }

    pub fn get_key(&self) -> String {
        self.name.clone()
    }

    pub fn get(&self, name: &str) -> Option<usize> {
        self.defined
            .get(name)
            .or_else(|| self.imported.get(name))
            .copied()
    }

    pub fn use_module(&mut self, other: &Module) {
        for (key, index) in other.defined.iter() {
            self.imported.insert(key.clone(), *index);
        }
    }
}

/// This holds the running environment.
#[derive(Default, Clone)]
pub struct Environment {
    /// Constants that are literals that are referred to in the source code.
    /// This includes that lammbdas that words are built from.
    pub constants: Vec<Value>,

    /// The vector of instructions to execute.
    pub instructions: Vec<usize>,

    /// Operations that have been loaded into the environment. This is all of
    /// built-ins, rust-defined functions, and user-defined.
    pub op_table: Vec<Shared<Lambda>>,

    /// This holds the modules that have been loaded.
    pub modules: HashMap<String, Module>,
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

impl Environment {
    pub fn new() -> Self {
        Environment {
            constants: Vec::new(),
            instructions: Vec::new(),
            op_table: Vec::new(),
            modules: HashMap::new(),
        }
    }

    pub fn from_parameters(
        constants: Vec<Value>,
        instructions: Vec<usize>,
        op_table: Vec<Shared<Lambda>>,
        modules: HashMap<String, Module>,
    ) -> Self {
        Environment {
            constants,
            instructions,
            op_table,
            modules,
        }
    }

    pub fn with_builtins() -> Self {
        let mut env = Self::new();

        let op_table = create_op_table();
        env.set_op_table(op_table);

        let kernel = create_kernel_module();
        env.modules.insert(KERNEL.to_string(), kernel);

        env
    }

    /// Create a new module with a given name and import words from the
    /// kernel.
    pub fn create_module(&self, name: &str) -> Module {
        let kernel = &self.modules[KERNEL];
        Module::with_imports(name, kernel)
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

    pub fn add_to_op_table(
        &mut self,
        module_key: &str,
        lambda: Shared<Lambda>,
    ) -> CompilerResult<()> {
        log::trace!(
            "Environment::add_to_op_table {} {:?}",
            module_key,
            lambda.borrow().name
        );
        let index = self.op_table.len();
        self.op_table.push(lambda.clone());

        if let Some(name) = lambda.borrow().name.as_ref() {
            let module = self
                .get_module_mut(module_key)
                .ok_or_else(|| CompilerError::ModuleNotFound(module_key.to_string()))?;
            log::trace!(
                "Environment::add_to_op_table '{}::{}' => {}",
                module_key,
                name,
                index
            );
            module.defined.insert(name.clone(), index);
        }

        Ok(())
    }

    pub fn set_imported(&mut self, module_path: &str, imported: HashMap<String, usize>) {
        if let Some(module) = self.modules.get_mut(module_path) {
            module.imported = imported;
        }
    }

    pub fn get_op_table_size(&self) -> usize {
        self.op_table.len()
    }

    pub fn get_instructions(&self) -> &Vec<usize> {
        &self.instructions
    }

    pub fn get_op_name(&self, ip: usize) -> Option<String> {
        // TODO: name here is actually the path, which isn't ideal
        for (name, module) in self.modules.iter() {
            for (word, index) in module.defined.iter() {
                let lambda_ip = self.op_table.get(*index).and_then(|l| l.borrow().get_ip());
                if lambda_ip == Some(ip) {
                    // TODO: any way to use the Display implementation defined for Value or ValueData?
                    return Some(format!("{}::{}", name, word));
                }
            }
        }
        None
    }

    pub fn get_op_index(&self, module: &str, word: &str) -> Option<usize> {
        log::trace!("Environment::get_op_index {}::{}", module, word);
        self.modules.get(module).and_then(|m| {
            log::trace!("Environment::get_op_index module {}", m.name);
            m.get(word)
        })
    }

    pub fn get_instruction(&self, ip: usize) -> VMResult<usize> {
        self.instructions
            .get(ip)
            .copied()
            .ok_or_else(|| VMError::InvalidInstructionPointer(ip))
    }

    pub fn get_op(&self, ip: &usize, index: usize) -> VMResult<Shared<Lambda>> {
        self.op_table
            .get(index)
            .cloned()
            .ok_or_else(|| VMError::InvalidOpCode(*ip, index))
    }

    pub fn instructions_len(&self) -> usize {
        self.instructions.len()
    }

    pub fn get_module(&self, key: &str) -> Option<&Module> {
        self.modules.get(key)
    }

    pub fn get_module_mut(&mut self, key: &str) -> Option<&mut Module> {
        log::trace!("Environment::get_module_mut {:?}", key);
        self.modules.get_mut(key)
    }

    pub fn get_module_for(&self, scanner: &Scanner) -> Option<&Module> {
        self.modules.get(&scanner.source.get_key())
    }

    pub fn get_module_for_mut(&mut self, scanner: &Scanner) -> Option<&mut Module> {
        self.modules.get_mut(&scanner.source.get_key())
    }

    pub fn add_module(&mut self, module: Module) {
        let name = module.name.clone();
        self.modules.insert(name, module);
    }

    pub fn get_or_create_module_mut<'a>(&'a mut self, name: &str) -> &'a mut Module {
        log::trace!("Environment::get_or_create_module_mut {:?}", name);
        if self.modules.contains_key(name) {
            log::trace!(
                "Environment::get_or_create_module_mut get existing {:?}",
                name
            );
            self.modules.get_mut(name).unwrap()
        } else {
            log::trace!(
                "Environment::get_or_create_module_mut create new {:?}",
                name
            );
            let module = self.create_module(name);
            self.add_module(module);
            self.modules.get_mut(name).unwrap()
        }
    }

    pub fn use_module(&mut self, source: &str, dest: &str) -> Result<()> {
        let source = self
            .modules
            .get(source)
            .ok_or_else(|| CompilerError::ModuleNotFound(source.to_string()))?
            .defined
            .clone();
        let dest = self
            .modules
            .get_mut(dest)
            .ok_or_else(|| CompilerError::ModuleNotFound(dest.to_string()))?;
        for (key, index) in source {
            dest.imported.insert(key, index);
        }
        Ok(())
    }

    pub fn get_callable(&self, op_table_index: usize) -> Option<Shared<Lambda>> {
        self.op_table.get(op_table_index).cloned()
    }

    pub fn add_macro(&mut self, module_name: &str, macro_lambda: Lambda) -> Result<()> {
        log::trace!(
            "Environment::add_macro {}::{:?}",
            module_name,
            macro_lambda.name
        );
        let macro_lambda = shared(macro_lambda);
        self.add_to_op_table(module_name, macro_lambda.clone())?;
        Ok(())
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
            | OpCode::Stop
            | OpCode::Bye
            | OpCode::JumpStack
            | OpCode::Function
            | OpCode::PredeclareFunction
            | OpCode::ScanValue
            | OpCode::ScanValueList
            | OpCode::ScanObjectList
            | OpCode::LitStack
            | OpCode::Compile => self.debug_simple(op, f, ip),
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
        write!(f, " {:0>4}. {: <20} |", index, value)?;

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
        write!(f, "{: <20} | ", debugged)
    }

    fn write_call(&self, f: &mut fmt::Formatter<'_>, index: usize, name: &str) -> fmt::Result {
        write!(f, "{:0>4}. {: <14} | ", index, name)
    }

    fn write_function_names(&self, f: &mut fmt::Formatter<'_>, ip: usize) -> fmt::Result {
        let name = self.get_op_name(ip);

        // TODO: sometimes the column before this is omitted. Make them line up.
        if let Some(name) = name {
            write!(f, " {: <20} | ", name)?;
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
// impl Clone for Environment {
//     fn clone(&self) -> Self {
//         Environment {
//             constants: self.constants.clone(),
//             instructions: self.instructions.clone(),
//             op_table: self.op_table.clone(),
//             modules: self.modules.clone(),
//         }
//     }
// }
