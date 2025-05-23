use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::{Error, Result, VMError};
use crate::module::{Module, ModuleManager};
use crate::shared::{shared, Shared};
use crate::value::lambda::Lambda;
use crate::value::ValueData;
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const VECTORS: &str = "std/_vectors";

pub struct VectorsBuilder;

// TODO: really need to implement an Option type for here
impl InternalBuilder for VectorsBuilder {
    fn define_module(
        &self,
        _module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "<vector>", create_list);
        push_op(op_table, &mut index, "push!", push);
        push_op(op_table, &mut index, "push-left!", push_left);
        push_op(op_table, &mut index, "concat", concat);
        push_op(op_table, &mut index, "pop-left!", pop_left);
        push_op(op_table, &mut index, "pop!", pop);
        push_op(op_table, &mut index, "nth", nth);
        // TODO: second
        // TODO: third
        // TODO: last
        // TODO: set-nth!
        // TODO: length
        // TODO: in?
        // TODO: empty?
        // TODO: index-of?
        // TODO: subvector
        // TODO: join
        // TODO: sort!
        // TODO: map

        // TODO: load ./src/bootstrap/vectors.tardi

        Module {
            imported: HashMap::new(),
            path: None,
            name: VECTORS.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
}

// List operations
/// <vector> ( -- vec )
fn create_list(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    vm.push(shared(ValueData::List(Vec::new()).into()))
}

/// push! ( value vector -- )
fn push(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let value = vm.pop()?;

    (*list)
        .borrow_mut()
        .get_list_mut()
        .map(|l| l.push(value))
        .ok_or_else(|| VMError::TypeMismatch("push to list".to_string()))?;

    Ok(())
}

/// push-left! ( value vector -- )
fn push_left(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let value = vm.pop()?;

    (*list)
        .borrow_mut()
        .get_list_mut()
        .map(|l| l.insert(0, value))
        .ok_or_else(|| VMError::TypeMismatch("push-left to list".to_string()))?;

    Ok(())
}

/// concat ( list1 list2 -- list1+2 )
fn concat(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
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

/// pop-left! ( vector -- item )
fn pop_left(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let head = (*list)
        .borrow_mut()
        .get_list_mut()
        .ok_or_else(|| VMError::TypeMismatch(format!("pop-left! of list: {}", list.borrow())))
        .and_then(|l| {
            if l.is_empty() {
                Err(VMError::EmptyList)
            } else {
                Ok(l.remove(0))
            }
        })?;

    vm.push(head)
}

/// pop! ( vector -- item )
fn pop(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let item = list
        .borrow_mut()
        .get_list_mut()
        .ok_or_else(|| VMError::TypeMismatch("pop list".to_string()))
        .and_then(|l| l.pop().ok_or(VMError::EmptyList))?;
    vm.push(item)
}

/// nth ( i vector -- item/#f )
fn nth(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let index = vm
        .pop()?
        .borrow()
        .get_integer()
        .ok_or_else(|| VMError::TypeMismatch("nth index".to_string()))? as usize;
    let item = (*list)
        .borrow()
        .get_list()
        .ok_or_else(|| VMError::TypeMismatch("split head of list".to_string()))
        .map(|l| {
            l.get(index)
                .cloned()
                .unwrap_or_else(|| shared(false.into()))
        })?;

    vm.push(item)
}
