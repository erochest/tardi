use std::collections::{HashMap, HashSet};
use std::fs;

use crate::error::Result;
use crate::module::Module;
use crate::shared::shared;
use crate::vm::VM;
use crate::{compiler::Compiler, error::VMError};

use super::{push_op, InternalBuilder};

pub const FS: &str = "std/fs";

pub struct FsModule;
impl InternalBuilder for FsModule {
    fn define_module(
        &self,
        _module_manager: &crate::module::ModuleManager,
        op_table: &mut Vec<crate::shared::Shared<crate::value::lambda::Lambda>>,
    ) -> crate::module::Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "rm", rm);
        push_op(op_table, &mut index, "truncate", truncate);
        push_op(op_table, &mut index, "exists?", does_file_exist);
        // TODO: push_op(op_table, &mut index, "rmdir", rmdir);
        push_op(op_table, &mut index, "ensure-dir", ensure_dir);
        // TODO: push_op(op_table, &mut index, "touch", touch);
        // TODO: push_op(op_table, &mut index, "ls", ls);

        Module {
            imported: HashMap::new(),
            path: None,
            name: FS.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
}

/// path -- result-flag
fn rm(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("rm path must be string".to_string()))?;

    fs::remove_file(path)?;

    vm.push(shared(true.into()))?;
    Ok(())
}

/// path -- result-flag
fn truncate(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("truncate path must be string".to_string()))?;

    // TODO: propagate errors
    fs::write(path, "")?;

    vm.push(shared(true.into()))?;
    Ok(())
}

/// path -- ?
fn does_file_exist(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("exists? path must be string".to_string()))?;

    let exists = fs::exists(path)?;

    vm.push(shared(exists.into()))
}

/// path -- ?
/// Returns `#t` if it creates the directory, `#f` if not.
fn ensure_dir(vm: &mut VM, _compiler: &mut Compiler) -> Result<()> {
    let path = vm.pop()?;
    let path = path.borrow();
    let path = path
        .as_string()
        .ok_or_else(|| VMError::TypeMismatch("ensure-dir path must be string".to_string()))?;

    if fs::exists(path)? {
        vm.push(shared(false.into()))
    } else {
        fs::create_dir_all(path)?;
        vm.push(shared(true.into()))
    }
}
