use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::Result;
use crate::module::{Module, ModuleManager};
use crate::shared::Shared;
use crate::shared::{shared, unshare_clone};
use crate::value::lambda::Lambda;
use crate::value::{Value, ValueData};
use crate::vm::VM;

use super::{push_op_typed, InternalBuilder};

pub const SCANNING: &str = "std/scanning";

pub struct ScanningBuilder;
impl InternalBuilder for ScanningBuilder {
    fn define_module(
        &self,
        _module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();
        // These three have compiler-level effects and don't have a simple stack type.
        // We represent them with a polymorphic signature as a conservative approximation.
        push_op_typed(op_table, &mut index, "scan-value",        scan_value,        "( -- a )");
        push_op_typed(op_table, &mut index, "scan-value-list",   scan_value_list,   "( a -- vec )");
        push_op_typed(op_table, &mut index, "scan-object-list",  scan_object_list,  "( a -- vec )");
        // Optionally consume a `( ... )` type-signature block from the token stream.
        // If the next token is `(`, everything up to and including `)` is consumed
        // and discarded.  If not, the stream is left unchanged.
        // Stack effect: ( -- )  (pure scanner-level side-effect, no stack change)
        push_op_typed(op_table, &mut index, "<consume-type-sig>", consume_type_sig, "( -- )");
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

/// `<consume-type-sig>` ( S -- S )
///
/// Peeks at the next token in the source stream.  If it is `(`, scans and
/// discards all tokens up to and including the matching `)`.  Otherwise the
/// stream is left unchanged.  Either way, no value is pushed or popped.
fn consume_type_sig(_vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
    compiler.try_consume_type_sig()?;
    Ok(())
}
