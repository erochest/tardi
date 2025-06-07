use std::collections::HashMap;

use fs::{FsModule, FS};
use internals::{InternalsModule, INTERNALS};
use io::{IoModule, IO};
use kernel::{KernelModule, KERNEL};
use sandbox::{SandboxBuilder, SANDBOX};
use scanning::{ScanningBuilder, SCANNING};
use strings::{StringsBuilder, STRINGS};
use vectors::{VectorsBuilder, VECTORS};

use crate::compiler::error::CompilerError;
use crate::error::Result;
use crate::shared::{shared, Shared};
use crate::value::lambda::{Lambda, OpFn};
use crate::vm::VM;

use super::{Module, ModuleManager};

pub mod fs;
pub mod internals;
pub mod io;
pub mod kernel;
pub mod sandbox;
pub mod scanning;
pub mod strings;
pub mod vectors;

pub fn define_module(
    manager: &ModuleManager,
    name: &str,
    op_table: &mut Vec<Shared<Lambda>>,
) -> Result<Module> {
    let builder: Box<dyn InternalBuilder> = match name {
        FS => Box::new(FsModule),
        KERNEL => Box::new(KernelModule),
        INTERNALS => Box::new(InternalsModule),
        IO => Box::new(IoModule),
        SANDBOX => Box::new(SandboxBuilder),
        SCANNING => Box::new(ScanningBuilder),
        STRINGS => Box::new(StringsBuilder),
        VECTORS => Box::new(VectorsBuilder),
        _ => return Err(CompilerError::ModuleNotFound(name.to_string()).into()),
    };

    Ok(builder.define_module(manager, op_table))
}

trait InternalBuilder {
    fn define_module(
        &self,
        module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module;
}

fn push_op(
    op_table: &mut Vec<Shared<Lambda>>,
    table: &mut HashMap<String, usize>,
    name: &str,
    op: OpFn,
) {
    let lambda = Lambda::new_builtin(name, op);
    let index = op_table.len();
    op_table.push(shared(lambda));
    table.insert(name.to_string(), index);
}

fn push_macro(
    op_table: &mut Vec<Shared<Lambda>>,
    table: &mut HashMap<String, usize>,
    name: &str,
    op: OpFn,
) {
    let lambda = Lambda::new_builtin_macro(name, op);
    let index = op_table.len();
    op_table.push(shared(lambda));
    table.insert(name.to_string(), index);
}

pub fn push_true(vm: &mut VM) -> Result<()> {
    vm.push(shared(true.into()))
}

pub fn push_false(vm: &mut VM) -> Result<()> {
    vm.push(shared(false.into()))
}
