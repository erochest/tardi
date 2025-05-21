use std::collections::HashMap;

use crate::module::{Module, ModuleManager};
use crate::shared::Shared;
use crate::value::lambda::Lambda;

use super::InternalBuilder;

pub const VECTORS: &str = "std/vectors";

pub struct VectorsBuilder;

impl InternalBuilder for VectorsBuilder {
    fn define_module(
        &self,
        module_manager: &ModuleManager,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Module {
        let mut index = HashMap::new();
        Module {
            imported: HashMap::new(),
            path: None,
            name: VECTORS.to_string(),
            defined: index,
        }
    }
}
