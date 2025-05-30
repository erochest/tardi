use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::Result;
use crate::module::{Module, ModuleManager};
use crate::shared::Shared;
use crate::shared::{shared, unshare_clone};
use crate::value::lambda::Lambda;
use crate::value::{Value, ValueData};
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const SCANNING: &str = "std/scanning";

pub struct ScanningBuilder;
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
