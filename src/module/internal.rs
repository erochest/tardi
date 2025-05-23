use std::collections::{HashMap, HashSet};

use strings::{StringsBuilder, STRINGS};
use vectors::{VectorsBuilder, VECTORS};

use crate::compiler::error::CompilerError;
use crate::compiler::Compiler;
use crate::error::{Result, VMError};
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Lambda, OpFn};
use crate::value::{Value, ValueData, ValueVec};
use crate::vm::VM;

use super::{Module, ModuleManager, INTERNALS, KERNEL, SANDBOX, SCANNING};

pub mod strings;
pub mod vectors;

pub fn define_module(
    manager: &ModuleManager,
    name: &str,
    op_table: &mut Vec<Shared<Lambda>>,
) -> Result<Module> {
    let builder: Box<dyn InternalBuilder> = match name {
        KERNEL => Box::new(KernelModule),
        INTERNALS => Box::new(InternalsModule),
        SANDBOX => Box::new(SandboxBuilder),
        SCANNING => Box::new(ScanningBuilder),
        STRINGS => Box::new(StringsBuilder),
        VECTORS => Box::new(VectorsBuilder),
        _ => return Err(CompilerError::ModuleNotFound(name.to_string()).into()),
    };

    Ok(builder.define_module(manager, op_table))
}

trait InternalBuilder {
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
            exported: HashSet::new(),
        }
    }
}

struct ScanningBuilder;
impl InternalBuilder for ScanningBuilder {
    fn define_module(
        &self,
        _module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();
        push_op(op_table, &mut index, "scan-value", scan_value);
        push_op(op_table, &mut index, "scan-value-list", scan_value_list);
        push_op(op_table, &mut index, "scan-object-list", scan_object_list);
        // TODO: peek-value (for things like `inline` after function declarations)
        Module {
            imported: HashMap::new(),
            path: None,
            name: SCANNING.to_string(),
            defined: index,
            exported: HashSet::new(),
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
            exported: HashSet::new(),
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
        push_op(op_table, &mut index, "apply", apply);
        push_op(op_table, &mut index, "return", return_op);
        push_op(op_table, &mut index, "stop", stop);
        push_op(op_table, &mut index, "bye", bye);
        push_op(op_table, &mut index, "jump", jump);
        push_op(op_table, &mut index, "jump-stack", jump_stack);
        push_op(op_table, &mut index, "lit", lit_stack);
        push_op(op_table, &mut index, "compile", compile);
        push_macro(op_table, &mut index, "use:", use_module);
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

fn function(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.function()
}

fn predeclare_function(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.predeclare_function()
}

fn scan_value(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    let value = compiler.scan_word()?;
    let value = shared(value);
    vm.push(value)?;
    Ok(())
}

fn scan_value_list(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
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

fn scan_object_list(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    let delimiter = vm.pop()?;
    let delimiter: Value = unshare_clone(delimiter);
    let delimiter = delimiter.data;

    // call Compiler::scan_value_list
    let value = shared(Value::new(ValueData::List(vec![])));
    compiler.scan_object_list(vm, delimiter, value.clone())?;

    vm.push(value)?;

    Ok(())
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

fn use_module(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    compiler.use_module(vm)
}

fn export_list(vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
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
        .extend(names.iter().map(|v| v.get_word().unwrap().to_string()));

    Ok(())
}
