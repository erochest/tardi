use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::compiler::Compiler;
use crate::error::Result;
use crate::error::VMError;
use crate::module::Module;
use crate::shared::shared;
use crate::value::ValueData;
use crate::vm::VM;

use super::{push_op, InternalBuilder};

pub const IO: &str = "std/io";

pub struct IoModule;
impl InternalBuilder for IoModule {
    fn define_module(
        &self,
        module_manager: &crate::module::ModuleManager,
        op_table: &mut Vec<crate::shared::Shared<crate::value::lambda::Lambda>>,
    ) -> crate::module::Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "write-file", write_file);
        push_op(op_table, &mut index, "read-file", read_file);
        push_op(op_table, &mut index, "open", open);
        push_op(op_table, &mut index, "close", close);

        Module {
            imported: HashMap::new(),
            path: None,
            name: IO.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
}

fn write_file(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("write-file path must be string".to_string()))?;
    let contents = vm.pop()?;
    let contents = contents.borrow();
    let contents = contents
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("write-file contents must be string".to_string()))?;

    // TODO: needs to propagate errors
    fs::write(path, contents)?;

    vm.push(shared(true.into()))?;
    Ok(())
}

fn read_file(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("read-file path must be string".to_string()))?;

    // TODO: needs to propagate errors
    let contents = fs::read_to_string(path)?;

    vm.push(shared(contents.into()))?;
    vm.push(shared(true.into()))?;

    Ok(())
}

fn open(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let mode = vm.pop()?;
    let mode = mode.borrow();
    let mode = mode
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("open mode must be string".to_string()))?;
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .get_string()
        .ok_or_else(|| VMError::TypeMismatch("open path must be string".to_string()))?;
    let path = PathBuf::from_str(path)?;

    let mut open_options = fs::OpenOptions::new();
    // r,w,a,t
    if mode.contains('a') {
        open_options.append(true);
        open_options.create(true);
    }
    if mode.contains('r') {
        open_options.read(true);
    }
    if mode.contains('t') {
        open_options.truncate(true);
    }
    if mode.contains('w') {
        open_options.write(true);
        open_options.create(true);
    }

    let file = open_options.open(path.clone())?;
    let value_data = ValueData::File(path, mode.to_string(), shared(file));
    vm.push(shared(value_data.into()))?;

    Ok(())
}

fn close(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let file_value = vm.pop()?;
    let mut file_value = file_value.borrow_mut();
    let file_value = file_value
        .as_file_mut()
        .ok_or_else(|| VMError::TypeMismatch("close must be a file".to_string()))?;

    // TODO: propagate errors
    file_value.borrow_mut().sync_all()?;

    vm.push(shared(true.into()))?;

    Ok(())
}
