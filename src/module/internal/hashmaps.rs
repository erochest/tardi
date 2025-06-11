use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::Result;

use crate::module::{
    internal::{push_op, InternalBuilder},
    Module,
};
use crate::shared::shared;
use crate::value::ValueData;
use crate::vm::VM;

pub const HASHMAPS: &str = "std/_hashmaps";

pub struct HashMapsBuilder;

impl InternalBuilder for HashMapsBuilder {
    fn define_module(
        &self,
        module_manager: &crate::module::ModuleManager,
        op_table: &mut Vec<crate::shared::Shared<crate::value::lambda::Lambda>>,
    ) -> crate::module::Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "<hashmap>", hashmap);

        Module {
            imported: HashMap::new(),
            path: None,
            name: HASHMAPS.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
}

// <hashmap> ( -- hashmap )
fn hashmap(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let hashmap = HashMap::new();
    let value_data = ValueData::HashMap(hashmap);

    vm.push(shared(value_data.into()))?;

    Ok(())
}
