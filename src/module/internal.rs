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
use crate::module::internal::hashmaps::{HashMapsBuilder, HASHMAPS};
use crate::shared::{shared, Shared};
use crate::types::TypeSig;
use crate::value::lambda::{Lambda, OpFn};
use crate::vm::VM;

use super::{Module, ModuleManager};

pub mod fs;
pub mod hashmaps;
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
        HASHMAPS => Box::new(HashMapsBuilder),
        INTERNALS => Box::new(InternalsModule),
        IO => Box::new(IoModule),
        KERNEL => Box::new(KernelModule),
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

/// Register a built-in word with an associated type signature string.
///
/// The `type_sig` argument is a string like `"( S a -- S a a )"`.  Parse
/// errors are logged as warnings and the word is registered without a
/// signature rather than panicking.
fn push_op_typed(
    op_table: &mut Vec<Shared<Lambda>>,
    table: &mut HashMap<String, usize>,
    name: &str,
    op: OpFn,
    type_sig: &str,
) {
    let sig = type_sig.parse::<TypeSig>().ok();
    if sig.is_none() {
        log::warn!("push_op_typed: failed to parse type sig {:?} for {:?}", type_sig, name);
    }
    let lambda = Lambda::new_builtin(name, op).with_type_sig(sig.unwrap_or_else(|| {
        // Fallback: use an identity sig with no types
        "( S -- S )".parse().unwrap()
    }));
    let index = op_table.len();
    op_table.push(shared(lambda));
    table.insert(name.to_string(), index);
}

/// Register a built-in macro with an associated type signature string.
fn push_macro_typed(
    op_table: &mut Vec<Shared<Lambda>>,
    table: &mut HashMap<String, usize>,
    name: &str,
    op: OpFn,
    type_sig: &str,
) {
    let sig = type_sig.parse::<TypeSig>().ok();
    if sig.is_none() {
        log::warn!("push_macro_typed: failed to parse type sig {:?} for {:?}", type_sig, name);
    }
    let lambda = Lambda::new_builtin_macro(name, op).with_type_sig(sig.unwrap_or_else(|| {
        "( S -- S )".parse().unwrap()
    }));
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
