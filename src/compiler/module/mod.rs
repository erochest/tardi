use std::env;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

use crate::compiler::error::{CompilerError, CompilerResult};
use crate::{config::Config, error::Result};

pub const KERNEL: &str = "std/kernel";
pub const SANDBOX: &str = "std/sandbox";

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
