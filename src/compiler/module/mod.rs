use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::{env, fmt};

use crate::compiler::error::{CompilerError, CompilerResult};
use crate::{config::Config, error::Result};

pub const KERNEL: &str = "std/kernel";
pub const SANDBOX: &str = "std/sandbox";

#[derive(Default, Clone)]
pub struct Module {
    pub path: Option<PathBuf>,
    pub name: String,

    /// This maps a word name to its index in the environment's `op_table`.
    pub defined: HashMap<String, usize>,

    /// This maps the imported word names to their indexes in the environment's `op_table`.
    pub imported: HashMap<String, usize>,
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
        writeln!(f)?;
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

    pub fn use_module(&mut self, other: &Module) {
        for (key, index) in other.defined.iter() {
            self.imported.insert(key.clone(), *index);
        }
    }
}

#[derive(Debug)]
pub struct Loader {
    pub paths: Vec<PathBuf>,
}

// TODO: have Environment own this
// TODO: move modules out of Environment
// TODO: have this keep and return modules
// TODO: special handling for known internal modules
// TODO: std/internals
// TODO: std/scanning
// TODO: std/strings
// TODO: std/vectors
impl Loader {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Loader {
        let paths = Vec::from_iter(
            paths
                .iter()
                .filter_map(|p| p.as_ref().to_path_buf().canonicalize().ok()),
        );
        Loader { paths }
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
            let target = path.join(module);
            let target = target.with_extension("tardi");
            log::trace!("testing module at {:?}", target);
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
}

impl Default for Loader {
    fn default() -> Self {
        let current_dir = env::current_dir().unwrap();
        let current_dir = current_dir.canonicalize().unwrap();

        let paths = vec![current_dir];
        Loader { paths }
    }
}

impl From<Config> for Loader {
    fn from(config: Config) -> Self {
        Loader::from(&config)
    }
}

impl From<&Config> for Loader {
    fn from(_config: &Config) -> Self {
        todo!("Loader::from<&Config>")
    }
}

#[cfg(test)]
mod tests;
