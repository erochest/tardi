use std::collections::HashMap;

use internals::{InternalsModule, INTERNALS};
use kernel::{KernelModule, KERNEL};
use sandbox::{SandboxBuilder, SANDBOX};
use scanning::{ScanningBuilder, SCANNING};
use strings::{StringsBuilder, STRINGS};
use vectors::{VectorsBuilder, VECTORS};

use crate::compiler::error::CompilerError;
use crate::error::Result;
use crate::shared::{shared, Shared};
use crate::value::lambda::{Lambda, OpFn};

use super::{Module, ModuleManager};

pub mod internals;
pub mod kernel;
pub mod sandbox;
pub mod scanning;
pub mod strings;
pub mod vectors;

// TODO: break std/scanning of these into their own modules

pub fn define_module(
    manager: &ModuleManager,
    name: &str,
    op_table: &mut Vec<Shared<Lambda>>,
) -> Result<Module> {
    let builder: Box<dyn InternalBuilder> = match name {
        KERNEL => Box::new(KernelModule),
        INTERNALS => Box::new(InternalsModule),
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
