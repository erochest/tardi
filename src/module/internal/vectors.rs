use std::collections::HashMap;

use crate::compiler::Compiler;
use crate::error::{Error, Result, VMError};
use crate::module::{Module, ModuleManager};
use crate::shared::{shared, Shared};
use crate::value::lambda::Lambda;
use crate::value::ValueData;
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
    vm.push(shared(ValueData::List(Vec::new()).into()))
}

pub fn append(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let value = vm.pop()?;

    (*list)
        .borrow_mut()
        .get_list_mut()
        .map(|l| l.push(value))
        .ok_or_else(|| VMError::TypeMismatch("append to list".to_string()))?;

    Ok(())
}

pub fn prepend(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let value = vm.pop()?;

    (*list)
        .borrow_mut()
        .get_list_mut()
        .map(|l| l.insert(0, value))
        .ok_or_else(|| VMError::TypeMismatch("prepend to list".to_string()))?;

    Ok(())
}

pub fn concat(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list2 = vm.pop()?;
    let list1 = vm.pop()?;

    let new_items = {
        let list1 = list1.borrow();
        let list2 = list2.borrow();
        let list1_ref = list1.get_list();
        let list2_ref = list2.get_list();
        match (list1_ref, list2_ref) {
            (Some(items1), Some(items2)) => {
                let mut new_items = items1.clone();
                new_items.extend(items2.iter().cloned());
                Ok(new_items)
            }
            _ => Err(Error::from(VMError::TypeMismatch(
                "concatenate lists".to_string(),
            ))),
        }
    }?;

    vm.push(shared(ValueData::List(new_items).into()))
}

pub fn split_head(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let head = (*list)
        .borrow_mut()
        .get_list_mut()
        .ok_or_else(|| VMError::TypeMismatch("split head of list".to_string()))
        .and_then(|l| {
            if l.is_empty() {
                Err(VMError::EmptyList)
            } else {
                Ok(l.remove(0))
            }
        })?;

    vm.push(head)
}
