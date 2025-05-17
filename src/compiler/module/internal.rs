use std::collections::HashMap;

use crate::compiler::Compiler;
use crate::error::Result;
use crate::shared::{unshare_clone, Shared};
use crate::value::lambda::{Lambda, OpFn};
use crate::value::{Value, ValueData};
use crate::vm::VM;
use crate::{compiler::error::CompilerError, shared::shared};

use super::{Module, ModuleManager, INTERNALS, KERNEL, SANDBOX};

pub fn define_module(
    manager: &ModuleManager,
    name: &str,
    op_table: &mut Vec<Shared<Lambda>>,
) -> Result<Module> {
    let builder: Box<dyn InternalBuilder> = match name {
        KERNEL => Box::new(KernelModule),
        INTERNALS => Box::new(InternalsModule),
        SANDBOX => Box::new(SandboxBuilder),
        _ => return Err(CompilerError::ModuleNotFound(name.to_string()).into()),
    };

    Ok(builder.define_module(manager, op_table))
}

pub trait InternalBuilder {
    fn define_module(
        &self,
        module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module;
}

struct SandboxBuilder;
impl InternalBuilder for SandboxBuilder {
    fn define_module(
        &self,
        manager: &ModuleManager,
        _op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let imported = manager.get_kernel().defined.clone();
        let defined = HashMap::new();

        Module {
            imported,
            path: None,
            name: SANDBOX.to_string(),
            defined,
        }
    }
}

struct InternalsModule;
impl InternalBuilder for InternalsModule {
    fn define_module(
        &self,
        _manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "<function>", function);
        push_op(
            op_table,
            &mut index,
            "<predeclare-function>",
            predeclare_function,
        );

        Module {
            imported: HashMap::new(),
            path: None,
            name: INTERNALS.to_string(),
            defined: index,
        }
    }
}

struct KernelModule;
impl InternalBuilder for KernelModule {
    fn define_module(
        &self,
        _manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "<lit>", lit);
        push_op(op_table, &mut index, "dup", dup);
        push_op(op_table, &mut index, "swap", swap);
        push_op(op_table, &mut index, "rot", rot);
        push_op(op_table, &mut index, "drop", drop_op);
        push_op(op_table, &mut index, "clear", clear);
        push_op(op_table, &mut index, "stack-size", stack_size);
        push_op(op_table, &mut index, "+", add);
        push_op(op_table, &mut index, "-", subtract);
        push_op(op_table, &mut index, "*", multiply);
        push_op(op_table, &mut index, "/", divide);
        push_op(op_table, &mut index, "==", equal);
        push_op(op_table, &mut index, "<", less);
        push_op(op_table, &mut index, ">", greater);
        push_op(op_table, &mut index, "!", not);
        push_op(op_table, &mut index, "?", question);
        push_op(op_table, &mut index, ">r", to_r);
        push_op(op_table, &mut index, "r>", r_from);
        push_op(op_table, &mut index, "r@", r_fetch);
        push_op(op_table, &mut index, "create-list", create_list);
        push_op(op_table, &mut index, "append", append);
        push_op(op_table, &mut index, "prepend", prepend);
        push_op(op_table, &mut index, "concat", concat);
        push_op(op_table, &mut index, "split-head", split_head);
        push_op(op_table, &mut index, "<string>", create_string);
        push_op(op_table, &mut index, ">string", to_string);
        push_op(op_table, &mut index, "utf8>string", utf8_to_string);
        push_op(op_table, &mut index, "string-concat", string_concat);
        push_op(op_table, &mut index, "apply", apply);
        push_op(op_table, &mut index, "return", return_op);
        push_op(op_table, &mut index, "stop", stop);
        push_op(op_table, &mut index, "bye", bye);
        push_op(op_table, &mut index, "jump", jump);
        push_op(op_table, &mut index, "jump-stack", jump_stack);
        // TODO: std/scanning
        push_op(op_table, &mut index, "scan-value", scan_value);
        push_op(op_table, &mut index, "scan-value-list", scan_value_list);
        push_op(op_table, &mut index, "scan-object-list", scan_object_list);
        push_op(op_table, &mut index, "lit", lit_stack);
        push_op(op_table, &mut index, "compile", compile);
        push_macro(op_table, &mut index, "use:", use_module);

        Module {
            imported: HashMap::new(),
            path: None,
            name: KERNEL.to_string(),
            defined: index,
        }
    }
}

fn push_op(
    op_table: &mut Vec<Shared<Lambda>>,
    table: &mut HashMap<String, usize>,
    name: &str,
    op: OpFn,
) {
    let lambda = Lambda::new_builtin(name, op);
    let index = op_table.len();
    op_table.push(shared(lambda));
    table.insert(name.to_string(), index);
}

fn push_macro(
    op_table: &mut Vec<Shared<Lambda>>,
    table: &mut HashMap<String, usize>,
    name: &str,
    op: OpFn,
) {
    let lambda = Lambda::new_builtin_macro(name, op);
    let index = op_table.len();
    op_table.push(shared(lambda));
    table.insert(name.to_string(), index);
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
#[allow(dead_code)]
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
    let value = shared(Value::new(ValueData::List(vec![])));
    compiler.scan_object_list(vm, delimiter, value.clone())?;

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
