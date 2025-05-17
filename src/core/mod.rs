use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::module::{KERNEL, SANDBOX};
use crate::compiler::Compiler;
use crate::config::Config;
use crate::env::{Environment, Module};
use crate::error::Result;
use crate::scanner::error::ScannerError;
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Lambda, OpFn};
use crate::value::{Value, ValueData};
use crate::vm::{OpCode, VM};

pub trait Execute {
    fn run(&mut self, env: Shared<Environment>, compiler: &mut Compiler) -> Result<()>;
    fn stack(&self) -> Vec<Value>;
    fn execute_macro(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        trigger: &ValueData,
        lambda: &Lambda,
        token_buffer: Shared<Value>,
    ) -> Result<()>;
}

// TODO: make the VM the orchestrator and get rid of this?
pub struct Tardi {
    pub input: Option<String>,
    pub environment: Shared<Environment>,
    pub compiler: Compiler,
    pub executor: VM,
}

impl Tardi {
    pub fn assemble(environment: Environment, compiler: Compiler, executor: VM) -> Self {
        Tardi {
            input: None,
            environment: shared(environment),
            compiler,
            executor,
        }
    }

    // TODO: add bootstrapping to Config and then depreate this
    pub fn new(bootstrap_dir: Option<PathBuf>) -> Result<Self> {
        let mut tardi = Tardi::default();
        tardi.bootstrap(bootstrap_dir)?;
        Ok(tardi)
    }

    // TODO: internal modules that are defined on demand
    // TODO: std/strings
    // TODO: std/vectors
    pub fn bootstrap(&mut self, bootstrap_dir: Option<PathBuf>) -> Result<()> {
        if let Some(bootstrap_dir) = bootstrap_dir {
            log::trace!("Tardi::bootstrap {:?}", bootstrap_dir);
            if !bootstrap_dir.exists() {
                return Ok(());
            }
            let mut files = bootstrap_dir
                .read_dir()
                .unwrap()
                .filter_map(|dir_entry| dir_entry.ok())
                .map(|dir_entry| dir_entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "tardi"))
                .collect::<Vec<_>>();
            files.sort();
            for file in files {
                log::debug!("bootstrapping from {:?}", file);
                let input = fs::read_to_string(file)?;
                self.execute_str(&input)?;
            }
        } else {
            log::trace!("Tardi::bootstrap internal modules");
            self.execute_module_str(KERNEL, include_str!("../bootstrap/00-core-macros.tardi"))?;
            self.execute_module_str(KERNEL, include_str!("../bootstrap/01-stack-ops.tardi"))?;
            self.execute_module_str(KERNEL, include_str!("../bootstrap/02-core-ops.tardi"))?;
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.input = None;
    }

    pub fn compile_str(&mut self, module_name: &str, input: &str) -> Result<Shared<Environment>> {
        log::debug!("input : {}", input);
        self.compiler.compile_internal(
            &mut self.executor,
            self.environment.clone(),
            module_name,
            input,
        )?;
        Ok(self.environment.clone())
    }

    pub fn compile_script(&mut self, path: &Path) -> Result<Shared<Environment>> {
        self.compiler
            .compile_script(&mut self.executor, self.environment.clone(), path)?;
        Ok(self.environment.clone())
    }

    pub fn execute(&mut self) -> Result<()> {
        log::debug!("environment:\n{:?}", self.environment.borrow());
        self.executor
            .run(self.environment.clone(), &mut self.compiler)
    }

    pub fn execute_str(&mut self, input: &str) -> Result<()> {
        log::trace!("Tardi::execute_str");
        self.reset();
        self.compile_str(SANDBOX, input)?;
        self.execute()
    }

    pub fn execute_module_str(&mut self, module: &str, input: &str) -> Result<()> {
        log::trace!("Tardi::execute_module_str");
        self.reset();
        self.compile_str(module, input)?;
        self.execute()
    }

    pub fn execute_file(&mut self, path: &Path) -> Result<()> {
        log::trace!("Tardi::execute_file");
        self.reset();
        self.compile_script(path)?;
        self.execute()
    }

    pub fn stack(&self) -> Vec<Value> {
        self.executor.stack()
    }

    // allowing because this is used in tests
    // TODO: can i move this into the test module?
    #[allow(dead_code)]
    pub(crate) fn execute_ip(&mut self, ip: usize) -> Result<()> {
        let bookmark = self.executor.ip;
        self.executor.ip = ip;
        self.execute()?;
        self.executor.ip = bookmark;
        Ok(())
    }
}

// TODO: add bootstrapping from the default directory to here?
// seems like too much. all of this will be obvious defaults,
// but the bootstrap dir will be more often configured
impl Default for Tardi {
    fn default() -> Tardi {
        let environment = Environment::with_builtins();
        let compiler = Compiler::default();
        let executor = VM::new();
        Tardi::assemble(environment, compiler, executor)
    }
}

impl From<&Config> for Tardi {
    fn from(config: &Config) -> Self {
        // TODO: get bootstrapping from config as well
        let environment = Environment::with_builtins();
        let compiler = Compiler::from(config);
        let executor = VM::new();
        Tardi::assemble(environment, compiler, executor)
    }
}

// TODO: this needs to pay attention to modules and set up all
// internal modules.
/// Create the default operation table
pub fn create_op_table() -> Vec<Shared<Lambda>> {
    let size = OpCode::StringConcat as usize + 1;
    let mut op_table = Vec::with_capacity(size);

    // Set up the operation table
    push_op(&mut op_table, "<lit>", lit);
    push_op(&mut op_table, "dup", dup);
    push_op(&mut op_table, "swap", swap);
    push_op(&mut op_table, "rot", rot);
    push_op(&mut op_table, "drop", drop_op);
    push_op(&mut op_table, "clear", clear);
    push_op(&mut op_table, "stack-size", stack_size);
    push_op(&mut op_table, "+", add);
    push_op(&mut op_table, "-", subtract);
    push_op(&mut op_table, "*", multiply);
    push_op(&mut op_table, "/", divide);
    push_op(&mut op_table, "==", equal);
    push_op(&mut op_table, "<", less);
    push_op(&mut op_table, ">", greater);
    push_op(&mut op_table, "!", not);
    push_op(&mut op_table, "?", question);
    push_op(&mut op_table, ">r", to_r);
    push_op(&mut op_table, "r>", r_from);
    push_op(&mut op_table, "r@", r_fetch);
    push_op(&mut op_table, "create-list", create_list);
    push_op(&mut op_table, "append", append);
    push_op(&mut op_table, "prepend", prepend);
    push_op(&mut op_table, "concat", concat);
    push_op(&mut op_table, "split-head", split_head);
    push_op(&mut op_table, "<string>", create_string);
    push_op(&mut op_table, ">string", to_string);
    push_op(&mut op_table, "utf8>string", utf8_to_string);
    push_op(&mut op_table, "string-concat", string_concat);
    push_op(&mut op_table, "apply", apply);
    push_op(&mut op_table, "return", return_op);
    push_op(&mut op_table, "stop", stop);
    push_op(&mut op_table, "bye", bye);
    push_op(&mut op_table, "jump", jump);
    push_op(&mut op_table, "jump-stack", jump_stack);
    // TODO: move some of these into a std/internals module (& compile)
    push_op(&mut op_table, "<function>", function);
    push_op(&mut op_table, "<predefine-function>", predeclare_function);
    // TODO: std/scanning
    push_op(&mut op_table, "scan-value", scan_value);
    push_op(&mut op_table, "scan-value-list", scan_value_list);
    push_op(&mut op_table, "scan-object-list", scan_object_list);
    push_op(&mut op_table, "lit", lit_stack);
    push_op(&mut op_table, "compile", compile);
    push_macro(&mut op_table, "use:", use_module);

    op_table
}

pub fn create_kernel_module() -> Module {
    let op_table = create_op_table();
    let defined: HashMap<_, _> = op_table
        .iter()
        .enumerate()
        .map(|(index, lambda)| (lambda.borrow().name.clone().unwrap(), index))
        .collect();
    Module {
        imported: HashMap::new(),
        path: None,
        name: KERNEL.to_string(),
        defined,
    }
}

fn push_op(op_table: &mut Vec<Shared<Lambda>>, name: &str, op: OpFn) {
    let lambda = Lambda::new_builtin(name, op);
    op_table.push(shared(lambda));
}

fn push_macro(op_table: &mut Vec<Shared<Lambda>>, name: &str, op: OpFn) {
    let lambda = Lambda::new_builtin_macro(name, op);
    op_table.push(shared(lambda));
}

// Helper function to add an operation to the table and map
// Will be used when we implement function support
// fn add_word(op_table: &mut Vec<OpFn>, op_map: &mut HashMap<String, usize>, op: OpFn, name: &str) {
//     let index = op_table.len();
//     op_table.push(op);
//     op_map.insert(name.to_string(), index);
// }

// Define the operations
pub fn lit(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.lit()
}

pub fn dup(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.dup()
}

pub fn swap(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.swap()
}

pub fn rot(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.rot()
}

pub fn drop_op(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.drop_op()
}

pub fn clear(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.clear()
}

pub fn stack_size(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.stack_size_op()
}

pub fn add(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.add()
}

pub fn subtract(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.subtract()
}

pub fn multiply(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.multiply()
}

pub fn divide(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.divide()
}

pub fn to_r(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.to_r()
}

pub fn r_from(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.r_from()
}

pub fn r_fetch(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.r_fetch()
}

pub fn not(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.not()
}

pub fn question(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.question()
}

pub fn equal(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.equal()
}

pub fn less(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.less()
}

pub fn greater(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.greater()
}

// List operations
pub fn create_list(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.create_list()
}

pub fn append(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.append()
}

pub fn prepend(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.prepend()
}

pub fn concat(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.concat()
}

pub fn split_head(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.split_head()
}

// String operations
pub fn create_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.create_string()
}

pub fn to_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.to_string()
}

pub fn utf8_to_string(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.utf8_to_string()
}

pub fn string_concat(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.string_concat()
}

// Function operations
pub fn call(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    vm.call(compiler)
}

pub fn apply(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    vm.apply(compiler)
}

pub fn return_op(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.return_op()
}

pub fn stop(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.stop()
}

pub fn bye(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.bye()
}

pub fn jump(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.jump()
}

pub fn jump_stack(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.jump_stack()
}

pub fn function(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.function()
}

pub fn predeclare_function(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.predeclare_function()
}

pub fn scan_value(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    let value = compiler.scan_word()?;
    let value = shared(value);
    vm.push(value)?;
    Ok(())
}

pub fn scan_value_list(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    let delimiter = vm.pop()?;
    let delimiter: Value = unshare_clone(delimiter);
    let delimiter = &delimiter.data;

    let token_list = compiler.scan_value_list(delimiter)?;
    let list = token_list.into_iter().map(shared).collect();
    let value_data = ValueData::List(list);
    let value = Value::new(value_data);

    vm.push(shared(value))?;

    Ok(())
}

pub fn scan_object_list(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    let delimiter = vm.pop()?;
    let delimiter: Value = unshare_clone(delimiter);
    let delimiter = delimiter.data;

    // call Compiler::scan_value_list
    let env = vm.environment.clone().unwrap();
    let value = shared(Value::new(ValueData::List(vec![])));
    compiler.scan_object_list(vm, env, delimiter, value.clone())?;

    vm.push(value)?;

    Ok(())
}

pub fn lit_stack(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let value = vm.pop()?;
    let value: Value = unshare_clone(value);
    let literal = Value::new(ValueData::Literal(Box::new(value)));
    vm.push(shared(literal))?;

    Ok(())
}

pub fn compile(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    vm.compile(compiler)
}

pub fn use_module(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    compiler.use_module(vm)
}

#[cfg(test)]
mod tests;
