use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Deref;
use std::rc::Rc;

use crate::compiler::Compiler;
use crate::env::Environment;
use crate::error::{CompilerError, Error, Result, ScannerError, VMError};
use crate::scanner::Scanner;
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Callable, Lambda, OpFn};
use crate::value::{Value, ValueData};
use crate::vm::{OpCode, VM};

pub trait Scan {
    fn scan(&mut self, input: &str) -> Result<Vec<Result<Value>>>;
    fn set_source(&mut self, input: &str);
    fn scan_value(&mut self) -> Option<Result<Value>>;
    fn scan_values_until(&mut self, value_data: ValueData) -> Result<Vec<Result<Value>>>;
    fn read_string_until(&mut self, delimiter: &str) -> Result<String>;
}

pub trait Compile {
    fn compile<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        input: &str,
    ) -> Result<()>;
    fn compile_lambda<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        input: &str,
    ) -> Result<()>;
}

pub trait Execute {
    fn run(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        scanner: &mut Scanner,
    ) -> Result<()>;
    fn stack(&self) -> Vec<Value>;
    fn execute_macro(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        scanner: &mut Scanner,
        trigger: &ValueData,
        lambda: &Lambda,
        tokens: &[Value],
    ) -> Result<Vec<Value>>;
}

// TODO: make the VM the orchestrator and get rid of this?
pub struct Tardi {
    pub input: Option<String>,
    pub environment: Shared<Environment>,
    pub scanner: Scanner,
    pub compiler: Compiler,
    pub executor: VM,
}

impl Tardi {
    pub fn new(
        environment: Environment,
        scanner: Scanner,
        compiler: Compiler,
        executor: VM,
    ) -> Self {
        Tardi {
            input: None,
            environment: shared(environment),
            scanner,
            compiler,
            executor,
        }
    }

    pub fn reset(&mut self) {
        self.input = None;
    }

    pub fn scan_str(&mut self, input: &str) -> Result<Vec<Result<Value>>> {
        log::debug!("input : {:?}", input);
        let input = input.to_string();
        self.input = Some(input);
        Scan::scan(&mut self.scanner, self.input.as_ref().unwrap())
    }

    pub fn compile(&mut self, input: &str) -> Result<Shared<Environment>> {
        log::debug!("input : {}", input);
        self.compiler.compile(
            &mut self.executor,
            self.environment.clone(),
            &mut self.scanner,
            input,
        )?;
        Ok(self.environment.clone())
    }

    pub fn execute(&mut self) -> Result<()> {
        log::debug!("environment:\n{:?}", self.environment.borrow());
        self.executor.run(
            self.environment.clone(),
            &mut self.compiler,
            &mut self.scanner,
        )
    }

    pub fn execute_str(&mut self, input: &str) -> Result<()> {
        self.reset();
        self.compile(input)?;
        self.execute()
    }

    pub fn stack(&self) -> Vec<Value> {
        self.executor.stack()
    }

    pub(crate) fn execute_ip(&mut self, ip: usize) -> Result<()> {
        let bookmark = self.executor.ip;
        self.executor.ip = ip;
        self.execute()?;
        self.executor.ip = bookmark;
        Ok(())
    }
}

impl Default for Tardi {
    fn default() -> Tardi {
        let environment = Environment::with_builtins();
        let scanner = Scanner::default();
        let compiler = Compiler::default();
        let executor = VM::new();
        Tardi::new(environment, scanner, compiler, executor)
    }
}

// Create the default operation table
pub fn create_op_table() -> Vec<Shared<Lambda>> {
    let size = OpCode::StringConcat as usize + 1;
    let mut op_table = Vec::with_capacity(size);

    // Set up the operation table
    push_op(&mut op_table, "<lit>", lit);
    push_op(&mut op_table, "dup", dup);
    push_op(&mut op_table, "swap", swap);
    push_op(&mut op_table, "rot", rot);
    push_op(&mut op_table, "drop", drop_op);
    push_op(&mut op_table, "stack-size", stack_size);
    push_op(&mut op_table, "+", add);
    push_op(&mut op_table, "-", subtract);
    push_op(&mut op_table, "*", multiply);
    push_op(&mut op_table, "/", divide);
    push_op(&mut op_table, "==", equal);
    push_op(&mut op_table, "<", less);
    push_op(&mut op_table, ">", greater);
    push_op(&mut op_table, "!", not);
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
    push_op(&mut op_table, "call", call);
    push_op(&mut op_table, "call-stack", call_stack);
    push_op(&mut op_table, "return", return_op);
    push_op(&mut op_table, "jump", jump);
    push_op(&mut op_table, "jump-stack", jump_stack);
    push_op(&mut op_table, "<function>", function);
    push_op(&mut op_table, "scan-value", scan_value);
    push_op(&mut op_table, "scan-value-list", scan_value_list);
    push_op(&mut op_table, "scan-object-list", scan_object_list);
    push_op(&mut op_table, "lit", lit_stack);
    push_op(&mut op_table, "compile", compile);

    op_table
}

pub fn create_macro_table() -> HashMap<String, Lambda> {
    let mut map = HashMap::new();
    // insert_macro(&mut map, "scan-value-list", scan_token_list);
    map
}

fn push_op(op_table: &mut Vec<Shared<Lambda>>, name: &str, op: OpFn) {
    let lambda = Lambda::new_builtin(name, op);
    op_table.push(shared(lambda));
}

fn insert_macro(table: &mut HashMap<String, Lambda>, name: &str, op: OpFn) {
    let lambda = Lambda::new_builtin_macro(name, op);
    table.insert(name.to_string(), lambda);
}

// Helper function to add an operation to the table and map
// Will be used when we implement function support
// fn add_word(op_table: &mut Vec<OpFn>, op_map: &mut HashMap<String, usize>, op: OpFn, name: &str) {
//     let index = op_table.len();
//     op_table.push(op);
//     op_map.insert(name.to_string(), index);
// }

// Define the operations
pub fn lit(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.lit()
}

pub fn dup(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.dup()
}

pub fn swap(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.swap()
}

pub fn rot(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.rot()
}

pub fn drop_op(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.drop_op()
}

pub fn stack_size(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.stack_size_op()
}

pub fn add(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.add()
}

pub fn subtract(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.subtract()
}

pub fn multiply(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.multiply()
}

pub fn divide(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.divide()
}

pub fn to_r(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.to_r()
}

pub fn r_from(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.r_from()
}

pub fn r_fetch(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.r_fetch()
}

pub fn not(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.not()
}

pub fn equal(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.equal()
}

pub fn less(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.less()
}

pub fn greater(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.greater()
}

// List operations
pub fn create_list(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.create_list()
}

pub fn append(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.append()
}

pub fn prepend(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.prepend()
}

pub fn concat(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.concat()
}

pub fn split_head(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.split_head()
}

// String operations
pub fn create_string(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.create_string()
}

pub fn to_string(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.to_string()
}

pub fn utf8_to_string(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.utf8_to_string()
}

pub fn string_concat(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.string_concat()
}

// Function operations
pub fn call(vm: &mut VM, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
    vm.call(compiler, scanner)
}

pub fn call_stack(vm: &mut VM, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
    vm.call_stack(compiler, scanner)
}

pub fn return_op(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.return_op()
}

pub fn jump(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.jump()
}

pub fn jump_stack(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.jump_stack()
}

pub fn function(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.function()
}

pub fn scan_value(vm: &mut VM, _compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
    let value = scanner
        .scan_value()
        .ok_or(ScannerError::UnexpectedEndOfInput)??;
    let value = shared(value);
    vm.push(value)?;
    Ok(())
}

pub fn scan_value_list(vm: &mut VM, _compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
    let delimiter = vm.pop()?;
    let delimiter: Value = unshare_clone(delimiter);
    let delimiter = &delimiter.data;

    let token_list = scanner.scan_value_list(delimiter)?;
    let list = token_list.into_iter().map(shared).collect();
    let value_data = ValueData::List(list);
    let value = Value::new(value_data);

    vm.push(shared(value))?;

    Ok(())
}

pub fn scan_object_list(vm: &mut VM, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
    let delimiter = vm.pop()?;
    let delimiter: Value = unshare_clone(delimiter);
    let delimiter = delimiter.data;

    // call Compiler::scan_value_list
    let env = vm.environment.clone().unwrap();
    let values = compiler.scan_value_list(vm, env, delimiter, scanner)?;
    let list = values.into_iter().map(shared).collect();
    let value_data = ValueData::List(list);
    let value = Value::new(value_data);

    vm.push(shared(value))?;

    Ok(())
}

pub fn lit_stack(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    let value = vm.pop()?;
    let value: Value = unshare_clone(value);
    let literal = Value::new(ValueData::Literal(Box::new(value)));
    vm.push(shared(literal))?;

    Ok(())
}

pub fn compile(vm: &mut VM, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
    vm.compile(compiler, scanner)
}
