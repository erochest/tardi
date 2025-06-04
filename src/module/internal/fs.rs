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
        module_manager: &crate::module::ModuleManager,
        op_table: &mut Vec<crate::shared::Shared<crate::value::lambda::Lambda>>,
    ) -> crate::module::Module {
        let mut index = HashMap::new();

        push_op(op_table, &mut index, "rm", rm);
        push_op(op_table, &mut index, "truncate", truncate);
        // TODO: push_op(op_table, &mut index, "exists?", exists?);
        // TODO: push_op(op_table, &mut index, "mkdir", mkdir);
        // TODO: push_op(op_table, &mut index, "rmdir", rmdir);
        // TODO: push_op(op_table, &mut index, "ensure-dir", ensure-dir);
        // TODO: push_op(op_table, &mut index, "touch", touch);

        Module {
            imported: HashMap::new(),
            path: None,
            name: FS.to_string(),
            defined: index,
            exported: HashSet::new(),
        }
    }
}

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
