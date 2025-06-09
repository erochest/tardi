use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::{env, fmt};

use internal::define_module;
use internal::fs::FS;
use internal::internals::INTERNALS;
use internal::io::IO;
use internal::kernel::KERNEL;
use internal::sandbox::SANDBOX;
use internal::scanning::SCANNING;
use internal::strings::STRINGS;
use internal::vectors::VECTORS;
use lazy_static::lazy_static;

use crate::compiler::error::{CompilerError, CompilerResult};
use crate::shared::Shared;
use crate::value::lambda::Lambda;
use crate::{config::Config, error::Result};

pub mod internal;

lazy_static! {
    static ref INTERNAL_MODULES: HashSet<String> = vec![
        FS.to_string(),
        KERNEL.to_string(),
        SANDBOX.to_string(),
        INTERNALS.to_string(),
        IO.to_string(),
        SCANNING.to_string(),
        STRINGS.to_string(),
        VECTORS.to_string(),
    ]
    .into_iter()
    .collect();
}

#[derive(Default, Clone)]
pub struct Module {
    pub path: Option<PathBuf>,
    pub name: String,

    /// This maps a word name to its index in the environment's `op_table`.
    pub defined: HashMap<String, usize>,

    /// This maps the imported word names to their indexes in the environment's `op_table`.
    pub imported: HashMap<String, usize>,

    /// If listed, these are names that will be exported from either imported or defined.
    /// If not given, this is just defined.
    pub exported: HashSet<String>,
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "MODULE  : {} / {}",
            self.name,
            self.path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        )?;
        for (name, index) in self.defined.iter() {
            writeln!(f, "\tDEFINED : {:20} => {}", name, index)?;
        }
        for (name, index) in self.imported.iter() {
            writeln!(f, "\tIMPORTED: {:20} => {}", name, index)?;
        }
        for name in self.exported.iter() {
            writeln!(f, "\tEXPORTED: {:20} => {}", name, name)?;
        }
        Ok(())
    }
}

impl Module {
    pub fn new(name: &str) -> Module {
        let name = name.to_string();
        Module {
            path: None,
            name,
            defined: HashMap::new(),
            imported: HashMap::new(),
            exported: HashSet::new(),
        }
    }

    pub fn with_path(path: &Path, name: &str) -> Module {
        let path = Some(path.to_path_buf());
        let name = name.to_string();
        Module {
            path,
            name,
            defined: HashMap::new(),
            imported: HashMap::new(),
            exported: HashSet::new(),
        }
    }

    pub fn with_imports(name: &str, module: &Module) -> Module {
        let name = name.to_string();
        let imported = module.defined.clone();
        Module {
            path: None,
            name,
            defined: HashMap::new(),
            imported,
            exported: HashSet::new(),
        }
    }

    pub fn get_key(&self) -> String {
        self.name.clone()
    }

    pub fn get(&self, name: &str) -> Option<usize> {
        self.defined
            .get(name)
            .or_else(|| self.imported.get(name))
            .copied()
    }

    pub fn get_exports(&self) -> HashMap<String, usize> {
        let mut exports = HashMap::new();

        if self.exported.is_empty() {
            for (key, index) in self.defined.iter() {
                exports.insert(key.clone(), *index);
            }
        } else {
            for name in self.exported.iter() {
                let index = self.defined.get(name).or_else(|| self.imported.get(name));
                if let Some(index) = index {
                    exports.insert(name.clone(), *index);
                }
            }
        }

        exports
    }
}

#[derive(Debug, Clone)]
pub struct ModuleManager {
    /// This holds the search paths for loading new modules.
    pub paths: Vec<PathBuf>,

    /// This holds the modules that have been loaded.
    pub modules: HashMap<String, Module>,
}

impl ModuleManager {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> ModuleManager {
        let paths = Vec::from_iter(
            paths
                .iter()
                .filter_map(|p| p.as_ref().to_path_buf().canonicalize().ok()),
        );
        ModuleManager {
            paths,
            modules: HashMap::new(),
        }
    }

    pub fn load_kernel(&mut self, op_table: &mut Vec<Shared<Lambda>>) -> Result<()> {
        let kernel = define_module(self, KERNEL, op_table)?;
        self.modules.insert(KERNEL.to_string(), kernel);
        Ok(())
    }

    pub fn get_kernel(&self) -> &Module {
        &self.modules[KERNEL]
    }

    pub fn get_module_mut(&mut self, name: &str) -> Option<&mut Module> {
        self.modules.get_mut(name)
    }

    pub fn iter_modules(&self) -> impl Iterator<Item = &Module> {
        self.modules.values()
    }

    pub fn debug_module(&self, name: &str) {
        log::trace!(
            "modules loaded {:?}",
            self.modules.keys().collect::<Vec<_>>()
        );
        log::trace!(
            "words imported in   {}: {:?}",
            name,
            self.modules
                .get(name)
                .map_or_else(Default::default, |m| m.imported.keys().collect::<Vec<_>>())
        );
        log::trace!(
            "words defined  in   {}: {:?}",
            name,
            self.modules
                .get(name)
                .map_or_else(Default::default, |m| m.defined.keys().collect::<Vec<_>>())
        );
        log::trace!(
            "words exported from {}: {:?}",
            name,
            self.modules
                .get(name)
                .map_or_else(Default::default, |m| m.exported.iter().collect::<Vec<_>>())
        );
    }

    pub fn get_op_index(&self, module: &str, word: &str) -> Option<usize> {
        log::trace!("ModuleManager::get_op_index {}@{}", module, word);
        self.debug_module(module);
        self.modules.get(module).and_then(|m| m.get(word))
    }

    pub fn get(&self, module_name: &str) -> Option<&Module> {
        self.modules.get(module_name)
    }

    pub fn get_mut(&mut self, module_name: &str) -> Option<&mut Module> {
        self.modules.get_mut(module_name)
    }

    pub fn add_module(&mut self, module: Module) {
        let name = module.name.clone();
        self.modules.insert(name, module);
    }

    pub fn contains_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    pub fn load_internal(
        &mut self,
        name: &str,
        op_table: &mut Vec<Shared<Lambda>>,
    ) -> Result<&Module> {
        let module = define_module(self, name, op_table)?;
        log::trace!("ModuleManager::load_internal {}", name);
        log::trace!("{:?}", module);
        self.add_module(module);
        Ok(self.get(name).unwrap())
    }

    pub fn find(&self, module: &str, context: Option<&Path>) -> Result<Option<(String, PathBuf)>> {
        log::debug!("finding module '{}' in context {:?}", module, context);
        if module.starts_with("./") || module.starts_with("../") {
            if let Some(context) = context {
                return self
                    .find_abs_module(context, module)
                    .map_err(|err| err.into());
            } else {
                return Err(CompilerError::ModuleNotFound(module.to_string()).into());
            }
        }

        for path in self.paths.iter() {
            log::debug!(
                "finding module {} (context {:?}) in {}",
                module,
                context,
                path.display()
            );
            let target = path.join(module);
            let target = target.with_extension("tardi");
            if target.exists() {
                let target = target.canonicalize()?;
                return Ok(Some((module.to_string(), target)));
            }
        }

        Ok(None)
    }

    /// Takes a source module and a relative target module.
    /// It returns the absolute name and path to the target.
    ///
    /// The module has to be found under one of the search
    /// directories.
    pub fn find_abs_module(
        &self,
        source_module_path: &Path,
        target_module: &str,
    ) -> CompilerResult<Option<(String, PathBuf)>> {
        let target = source_module_path
            .parent()
            .ok_or_else(|| CompilerError::InvalidModulePath(source_module_path.to_owned()))?
            .join(target_module)
            .with_extension("tardi");
        let target = target.canonicalize();

        if target.is_err() {
            return Ok(None);
        }

        let target = target.unwrap();
        for path in self.paths.iter() {
            log::debug!(
                "checking for module {} ({}) in {}",
                target_module,
                source_module_path.display(),
                path.display()
            );
            if let Ok(suffix) = target.strip_prefix(path) {
                if let Some(name) = suffix.file_stem() {
                    let name = name.to_string_lossy();
                    let name = name.replace("\\", "/");
                    return Ok(Some((name, target)));
                }
            }
        }

        Err(CompilerError::InvalidModulePath(target))
    }

    /// Is this module defined through rust functions?
    pub fn is_internal(&self, name: &str) -> bool {
        INTERNAL_MODULES.contains(name)
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        let current_dir = env::current_dir().unwrap();
        let current_dir = current_dir.canonicalize().unwrap();

        let paths = vec![current_dir];
        ModuleManager {
            paths,
            modules: HashMap::default(),
        }
    }
}

impl From<Config> for ModuleManager {
    fn from(config: Config) -> Self {
        ModuleManager::from(&config)
    }
}

impl From<&Config> for ModuleManager {
    fn from(config: &Config) -> Self {
        ModuleManager::new(&config.module_path)
    }
}

#[cfg(test)]
mod tests;
