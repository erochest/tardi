use std::collections::{HashMap, HashSet};

use kernel::{KernelModule, KERNEL};
use strings::{StringsBuilder, STRINGS};
use vectors::{VectorsBuilder, VECTORS};

use crate::compiler::error::CompilerError;
use crate::compiler::Compiler;
use crate::error::Result;
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Lambda, OpFn};
use crate::value::{Value, ValueData};
use crate::vm::VM;

use super::{Module, ModuleManager, INTERNALS, SANDBOX, SCANNING};

pub mod kernel;
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
