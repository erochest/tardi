use std::env;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::error::CompilerError;
use crate::{config::Config, error::Result};

#[derive(Debug)]
pub struct Loader {
    pub paths: Vec<PathBuf>,
}

impl Loader {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Loader {
        let paths = Vec::from_iter(paths.iter().map(|p| p.as_ref().to_path_buf()));
        Loader { paths }
    }

    pub fn find(&self, module: &str, context: Option<&Path>) -> Result<Option<PathBuf>> {
        log::debug!("finding module '{}' in context {:?}", module, context);
        if module.starts_with("./") || module.starts_with("../") {
            if let Some(context) = context {
                return self.find_relative(module, context);
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
                return Ok(Some(target));
            }
        }

        Ok(None)
    }

    fn find_relative(&self, module: &str, context: &Path) -> Result<Option<PathBuf>> {
        let context = context.parent().unwrap();
        let target = context.join(module);
        let target = target.with_extension("tardi");
        log::trace!("testing relative module at {:?}", target);

        if target.exists() {
            let target = target.canonicalize()?;
            return Ok(Some(target));
        }

        Ok(None)
    }
}

impl Default for Loader {
    fn default() -> Self {
        let current_dir = env::current_dir().unwrap();
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
    fn from(config: &Config) -> Self {
        todo!("Loader::from<&Config>")
    }
}

#[cfg(test)]
mod tests;
