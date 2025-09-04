use std::collections::{HashMap, HashSet};

use crate::{
    module::{Module, ModuleManager},
    shared::Shared,
    value::lambda::Lambda,
};

use super::InternalBuilder;

// TODO: need to rename this. it conflicts with sandboxing a VM
pub const SANDBOX: &str = "std/sandbox";

pub struct SandboxBuilder;
impl InternalBuilder for SandboxBuilder {
    fn define_module(
        &self,
        manager: &ModuleManager,
        _op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let imported = manager.get_kernel().defined.clone();
        let defined = HashMap::new();

        Module {
            imported,
            path: None,
            name: SANDBOX.to_string(),
            defined,
            exported: HashSet::new(),
        }
    }
}
