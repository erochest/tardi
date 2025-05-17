use crate::compiler::error::CompilerError;
use crate::env::Environment;
use crate::error::Result;
use crate::shared::Shared;

use super::{Module, INTERNALS};

pub trait InternalBuilder {
    fn define_module(&self, env: Shared<Environment>) -> Module;
}

struct InternalsModule;
impl InternalBuilder for InternalsModule {
    fn define_module(&self, env: Shared<Environment>) -> Module {
        todo!("InternalsModule::define_module")
    }
}

pub fn define_module(name: &str, env: Shared<Environment>) -> Result<Module> {
    let builder: Box<dyn InternalBuilder> = match name {
        INTERNALS => Box::new(InternalsModule),
        _ => return Err(CompilerError::ModuleNotFound(name.to_string()).into()),
    };

    Ok(builder.define_module(env))
}
