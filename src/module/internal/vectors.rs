use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::{Error, Result, VMError};
use crate::module::{Module, ModuleManager};
use crate::shared::{shared, Shared};
use crate::value::lambda::Lambda;
use crate::value::{Value, ValueData};
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
        push_op(op_table, &mut index, "set-nth!", set_nth);
        push_op(op_table, &mut index, "length", length);
        push_op(op_table, &mut index, "in?", is_in);
        push_op(op_table, &mut index, "index-of?", index_of);
        push_op(op_table, &mut index, "subvector", subvector);
        push_op(op_table, &mut index, "join", join);
        push_op(op_table, &mut index, "sort!", sort);

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
        .as_list_mut()
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
        .as_list_mut()
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
        let list1_ref = list1.as_list();
        let list2_ref = list2.as_list();
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
        .as_list_mut()
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
        .as_list_mut()
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
        .as_integer()
        .ok_or_else(|| VMError::TypeMismatch("nth index".to_string()))? as usize;
    let item = (*list)
        .borrow()
        .as_list()
        .ok_or_else(|| VMError::TypeMismatch("nth list".to_string()))
        .map(|l| {
            l.get(index)
                .cloned()
                .unwrap_or_else(|| shared(false.into()))
        })?;

    vm.push(item)
}

/// set-nth! ( x i vector -- )
fn set_nth(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let index =
        vm.pop()?
            .borrow()
            .as_integer()
            .ok_or_else(|| VMError::TypeMismatch("set-nth! index".to_string()))? as usize;
    let item = vm.pop()?;
    let mut list = list.borrow_mut();
    let list = list
        .as_list_mut()
        .ok_or_else(|| VMError::TypeMismatch("set-nth! of list".to_string()))?;

    list[index] = item.clone();

    Ok(())
}

/// length ( vector -- length )
fn length(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let popped = vm.pop()?;
    let list = popped.borrow();
    let list = list.as_list().ok_or_else(|| {
        VMError::TypeMismatch(format!("length of list: {}", popped.borrow().to_repr()))
    })?;

    let length = list.len();

    vm.push(shared((length as i64).into()))
}

/// in? ( item vector -- ? )
fn is_in(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let popped = vm.pop()?;
    let list = popped.borrow();
    let list = list
        .as_list()
        .ok_or_else(|| VMError::TypeMismatch(format!("in? list: {}", popped.borrow().to_repr())))?;
    let item = vm.pop()?;

    let is_in = list.contains(&item);

    vm.push(shared(is_in.into()))
}

/// index-of? ( item vector -- i/#f )
fn index_of(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let list = list.borrow();
    let list = list
        .as_list()
        .ok_or_else(|| VMError::TypeMismatch("in? list".to_string()))?;
    let item = vm.pop()?;

    let index = list
        .iter()
        .position(|i| i == &item)
        .map(|i| Value::from(i as i64))
        .unwrap_or_else(|| Value::from(false));

    vm.push(shared(index))
}

/// subvector ( from to vector -- vector' )
fn subvector(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let list = vm.pop()?;
    let list = list.borrow();
    let list = list
        .as_list()
        .ok_or_else(|| VMError::TypeMismatch("subvector list".to_string()))?;
    let to_index =
        vm.pop()?
            .borrow()
            .as_integer()
            .ok_or_else(|| VMError::TypeMismatch("subvector to".to_string()))? as usize;
    let to_index = to_index.min(list.len());
    let from_index =
        vm.pop()?
            .borrow()
            .as_integer()
            .ok_or_else(|| VMError::TypeMismatch("subvector from".to_string()))? as usize;

    // TODO: be more defensive about to_index and from_index
    let subvector = list[from_index..to_index].to_vec();

    vm.push(shared(subvector.into()))
}

/// join ( vector glue -- string )
fn join(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let glue = vm.pop()?;
    let glue = glue.borrow();
    let glue = glue
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("join glue".to_string()))?;
    let list = vm.pop()?;
    let list = list.borrow();
    let list = list
        .as_list()
        .ok_or_else(|| VMError::TypeMismatch("join list".to_string()))?;

    let output = list
        .iter()
        // TODO: this works for non-strings, but strings can us an `id` function
        .map(|item| {
            item.borrow()
                .as_string()
                .map(|i| i.to_string())
                .unwrap_or_else(|| item.borrow().to_string())
        })
        .collect::<Vec<_>>()
        .join(glue);

    vm.push(shared(output.into()))
}

/// sort! ( vector -- )
fn sort(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let popped = vm.pop()?;
    // let repr = popped.borrow().to_repr();
    let mut list = popped.borrow_mut();
    if let Some(list) = list.as_list_mut() {
        list.sort();
    } else {
        // return Err(VMError::TypeMismatch(format!("sort list: {}", repr)).into());
        return Err(VMError::TypeMismatch("sort list".to_string()).into());
    }

    Ok(())
}
