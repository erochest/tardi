use std::collections::HashMap;

use crate::compiler::Compiler;
use crate::error::Result;
use crate::module::{Module, ModuleManager};
use crate::shared::Shared;
use crate::value::lambda::Lambda;
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const VECTORS: &str = "std/vectors";

pub struct VectorsBuilder;

impl InternalBuilder for VectorsBuilder {
    fn define_module(
        &self,
        module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "<vector>", create_list);
        push_op(op_table, &mut index, "append", append);
        push_op(op_table, &mut index, "prepend", prepend);
        push_op(op_table, &mut index, "concat", concat);
        push_op(op_table, &mut index, "split-head!", split_head);
        // TODO: pop
        // TODO: push (rename append)
        // TODO: pop_left
        // TODO: push_left
        // TODO: nth
        // TODO: length
        // TODO: in?
        // TODO: empty?
        // TODO: index-of?
        // TODO: subvector
        // TODO: join
        // TODO: sort
        // TODO: map

        Module {
            imported: HashMap::new(),
            path: None,
            name: VECTORS.to_string(),
            defined: index,
        }
    }
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
