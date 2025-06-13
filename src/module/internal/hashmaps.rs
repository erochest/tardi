use std::collections::{HashMap, HashSet};

use crate::compiler::Compiler;
use crate::error::{Result, VMError};

use crate::module::{
    internal::{push_op, InternalBuilder},
    Module,
};
use crate::shared::{shared, unshare_clone};
use crate::value::{Value, ValueData};
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

// >hashmap ( vector-of-pairs -- hashmap )
fn to_hashmap(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let vector = vm.pop()?;
    let vector = vector.borrow();
    let vector = vector
        .as_list()
        .ok_or_else(|| VMError::TypeMismatch(">hashmap expects a vector".to_string()))?;

    let mut hashmap = HashMap::new();
    for pair in vector.iter() {
        let pair = pair.borrow();
        let pair = pair.as_list().ok_or_else(|| {
            VMError::TypeMismatch(">hashmap expects a vector of vector pairs".to_string())
        })?;
        let key = pair
            .first()
            .ok_or_else(|| VMError::TypeMismatch("vector pair too short".to_string()))?;
        let key = unshare_clone(key.clone());
        let value = pair
            .get(1)
            .ok_or_else(|| VMError::TypeMismatch("vector pair too short".to_string()))?;

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
