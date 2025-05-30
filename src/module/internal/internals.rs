use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::Result;
use crate::module::{Module, ModuleManager};
use crate::shared::Shared;
use crate::value::lambda::Lambda;
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const INTERNALS: &str = "std/_internals";

pub struct InternalsModule;
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

fn function(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.function()
}

fn predeclare_function(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.predeclare_function()
}
