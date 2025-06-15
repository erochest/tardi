use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::{Result, VMError};

use crate::module::internal::{push_false, push_true};
use crate::module::{
    internal::{push_op, InternalBuilder},
    Module,
};
use crate::shared::{shared, unshare_clone};
use crate::value::{SharedValue, Value, ValueData};
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
        push_op(op_table, &mut index, ">hashmap", to_hashmap);
        push_op(op_table, &mut index, ">vector", to_vector);
        push_op(op_table, &mut index, "is-hashmap?", is_hashmap);
        push_op(op_table, &mut index, "length", length);
        push_op(op_table, &mut index, "get", get);
        push_op(op_table, &mut index, "add!", add);

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

    vm.push(shared(value_data.into()))
}

fn parse_vector_pair(word: &str, pair: &SharedValue) -> Result<(SharedValue, SharedValue)> {
    let pair = pair.borrow();
    let pair = pair.as_list().ok_or_else(|| {
        VMError::TypeMismatch(format!("{} expects a vector of vector pairs", word))
    })?;

    let key = pair
        .first()
        .ok_or_else(|| VMError::TypeMismatch("vector pair too short".to_string()))?;

    let value = pair
        .get(1)
        .ok_or_else(|| VMError::TypeMismatch("vector pair too short".to_string()))?;

    Ok((key.clone(), value.clone()))
}

// >hashmap ( vector-of-pairs -- hashmap )
fn to_hashmap(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let vector = vm.pop()?;
    let vector = vector.borrow();
    let vector = vector
        .as_list()
        .ok_or_else(|| VMError::TypeMismatch(">hashmap expects a vector".to_string()))?;

    let mut hashmap = HashMap::new();
    for pair in vector.iter() {
        let (key, value) = parse_vector_pair(">hashmap", pair)?;
        let key = unshare_clone(key.clone());

        hashmap.insert(key.data, value.clone());
    }

    let value_data = ValueData::HashMap(hashmap);
    vm.push(shared(value_data.into()))
}

// >vector ( hashmap -- vector-of-pairs )
fn to_vector(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;
    let object = object.borrow();
    let hashmap = object
        .data
        .as_hash_map()
        .ok_or_else(|| VMError::TypeMismatch("hashmaps/>vector expects a hashmap".to_string()))?;

    let vector = hashmap
        .iter()
        .map(|(k, v)| Value::from(vec![shared(Value::new(k.clone())), v.clone()]))
        .collect::<Vec<_>>();
    vm.push(shared(Value::from(vector)))
}

// is-hashmap? ( object -- ? )
fn is_hashmap(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;
    let object = object.borrow();
    let result = object.data.is_hash_map();
    vm.push(shared(result.into()))
}

// length ( hashmap -- int )
fn length(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let object = vm.pop()?;
    let object = object.borrow();
    let hashmap = object
        .data
        .as_hash_map()
        .ok_or_else(|| VMError::TypeMismatch("hashmaps/length expects a hashmap".to_string()))?;

    vm.push(shared((hashmap.len() as i64).into()))
}

// get ( key hashmap -- value ? )
fn get(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let popped = vm.pop()?;
    let popped = popped.borrow();
    let hashmap = popped.data.as_hash_map().ok_or_else(|| {
        VMError::TypeMismatch(format!(
            "hashmaps/get expects a hashmap: {}",
            popped.to_repr()
        ))
    })?;
    let popped = vm.pop()?;
    let key = popped.borrow();
    let key = &key.data;

    let value = hashmap.get(key);

    if let Some(value) = value {
        vm.push(value.clone())?;
        push_true(vm)
    } else {
        push_false(vm)?;
        push_false(vm)
    }
}

// add! ( pair hashmap -- )
fn add(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let popped = vm.pop()?;
    let mut hashmap = popped.borrow_mut();
    let hashmap = hashmap.data.as_hash_map_mut().ok_or_else(|| {
        VMError::TypeMismatch(format!(
            "hashmaps/add! expects a hashmap: {}",
            popped.borrow().to_repr()
        ))
    })?;

    let pair = vm.pop()?;
    let (key, value) = parse_vector_pair("add!", &pair)?;
    let key = key.borrow().data.clone();

    let _ = hashmap.insert(key, value);

    Ok(())
}
