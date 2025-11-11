use std::collections::HashMap;

use crate::module::Module;

#[derive(Debug, Default)]
pub struct Environment<'a> {
    modules: Vec<Module>,
    module_index: HashMap<String, &'a Module>,
}

impl<'a> Environment<'a> {
    pub fn create_module(&'a mut self, name: &str) -> &'a Module {
        let name = name.to_string();
        let module = Module::new(&name);
        self.modules.push(module);
        self.module_index
            .insert(name, self.modules.last().as_ref().unwrap());
        self.modules.last().as_ref().unwrap()
    }
}
