use std::collections::{HashMap, HashSet};

use crate::compiler::error::CompilerError;
use crate::compiler::Compiler;
use crate::error::{Result, VMError};
use crate::module::{Module, ModuleManager};
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::Lambda;
use crate::value::{Value, ValueData, ValueVec};
use crate::vm::VM;

use super::{push_macro, push_op, InternalBuilder};

mod loop_word;

pub const KERNEL: &str = "std/kernel";

pub struct KernelModule;
impl InternalBuilder for KernelModule {
    fn define_module(
        &self,
        _manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "<nop>", nop);
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
        push_op(op_table, &mut index, "apply", apply);
        push_op(op_table, &mut index, "return", return_op);
        push_op(op_table, &mut index, "stop", stop);
        push_op(op_table, &mut index, "bye", bye);
        push_op(op_table, &mut index, "jump", jump);
        push_op(op_table, &mut index, "jump-stack", jump_stack);
        push_op(op_table, &mut index, "lit", lit_stack);
        push_op(op_table, &mut index, "compile", compile);
        push_op(op_table, &mut index, "break", break_word);
        push_op(op_table, &mut index, "continue", continue_word);
        push_macro(op_table, &mut index, "loop", loop_word::loop_word);
        push_macro(op_table, &mut index, "uses:", use_module);
        push_macro(op_table, &mut index, "exports:", export_list);

        Module {
            imported: HashMap::new(),
            path: None,
            name: KERNEL.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
}

// Define the operations
fn nop(_vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    log::trace!("nop");
    Ok(())
}

fn lit(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.lit()
}

fn dup(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.dup()
}

fn swap(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.swap()
}

fn rot(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.rot()
}

fn drop_op(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.drop_op()
}

fn clear(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.clear()
}

fn stack_size(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.stack_size_op()
}

fn add(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.add()
}

fn subtract(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.subtract()
}

fn multiply(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.multiply()
}

fn divide(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.divide()
}

fn to_r(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.to_r()
}

fn r_from(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.r_from()
}

fn r_fetch(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.r_fetch()
}

fn not(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.not()
}

fn question(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.question()
}

fn equal(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.equal()
}

fn less(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.less()
}

fn greater(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.greater()
}

// Function operations
#[allow(dead_code)]
fn call(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    vm.call(compiler)
}

fn apply(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    vm.apply(compiler)
}

fn return_op(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.return_op()
}

fn stop(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.stop()
}

fn bye(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.bye()
}

fn jump(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.jump()
}

fn jump_stack(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.jump_stack()
}

fn lit_stack(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let value = vm.pop()?;
    let value: Value = unshare_clone(value);
    let literal = Value::new(ValueData::Literal(Box::new(value)));
    vm.push(shared(literal))?;

    Ok(())
}

fn compile(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    vm.compile(compiler)
}

fn break_word(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.clear_jump()
}

fn continue_word(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.clear_jump()
}

fn use_module(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    compiler.use_module(vm)
}

fn export_list(_vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    log::trace!("export_list");
    let names = compiler.scan_value_list(&ValueData::Word(";".to_string()))?;
    let module_name = compiler
        .current_scanner()
        .map(|s| s.source.get_key())
        .ok_or_else(|| CompilerError::InvalidState("missing scanner".to_string()))?;
    let env = compiler.environment()?;
    let mut env = env.borrow_mut();
    let module = env
        .get_module_mut(&module_name)
        .ok_or(VMError::MissingModule)?;
    log::trace!(
        "export_list setting exports for {}: {}",
        module_name,
        ValueVec(&names)
    );

    module
        .exported
        .extend(names.iter().map(|v| v.as_word().unwrap().to_string()));

    Ok(())
}
