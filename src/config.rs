use std::path::{Path, PathBuf};
use std::{env, fs};

use directories::ProjectDirs;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::value::{Dict, Map};
use figment::{Figment, Metadata, Profile, Provider};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repl: ReplConfig,
    pub module_path: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplConfig {
    pub edit_mode: EditMode,
    pub history_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditMode {
    Emacs,
    Vi,
}

impl From<EditMode> for rustyline::EditMode {
    fn from(val: EditMode) -> Self {
        match val {
            EditMode::Emacs => rustyline::EditMode::Emacs,
            EditMode::Vi => rustyline::EditMode::Vi,
        }
    }
}

impl Default for ReplConfig {
    fn default() -> Self {
        ReplConfig {
            edit_mode: EditMode::Emacs,
            history_file: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("", "", "Tardi");
        let global_module_path = project_dirs.as_ref().map(|pd| pd.data_dir());

        let current_dir = env::current_dir().unwrap();

        let mut paths = vec![current_dir];
        if let Some(global_module_path) = global_module_path {
            paths.insert(0, global_module_path.to_path_buf());
        }
        Config {
            repl: ReplConfig::default(),
            module_path: paths,
        }
    }
}

impl Config {
    pub fn try_from<T: Provider>(provider: T) -> Result<Self> {
        Figment::from(provider).extract().map_err(Error::from)
    }

    pub fn figment() -> Figment {
        use figment::providers::Env;
        Figment::from(Config::default()).merge(Env::prefixed("TARDI_"))
    }
}

impl From<Config> for rustyline::Config {
    fn from(val: Config) -> Self {
        let mut config = rustyline::Config::builder();
        config = config.edit_mode(val.repl.edit_mode.into());
        config.build()
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Tardi Library Configuration")
    }

    fn data(&self) -> std::result::Result<Map<Profile, Dict>, figment::Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }

    // TODO: make repl and script profiles?
    fn profile(&self) -> Option<Profile> {
        None
    }
}

fn default_config_file() -> Option<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", "Tardi");
    project_dirs
        .as_ref()
        .map(|pd| pd.config_dir().join("tardi.toml").to_owned())
}

// TODO: can I use the Cli struct as a provider?
// TODO: different qualifier and organization
/// This reads the configuration from a file and runs it through a standard
/// set of configuration locations.
///
/// 1. Defaults;
/// 2. The TOML configuration file passed in, which defaults to a platform-
///    appropriate value, if that exists;
/// 3. Environment variables that begin with "TARDI_".
///
/// Arguably this should be handled by the command-line executable, not
/// by the library, but often the VM needs to handle this consistently.
/// Plus, putting it here is more testable.
pub fn read_config_sources(config_file: &Option<&Path>) -> Result<Config> {
    // Config file is from CLI args or the standard platform configuration
    let config_file = config_file
        .as_ref()
        .map(|path| path.to_path_buf())
        .or_else(default_config_file);

    // read from the sources
    let mut figment = Figment::from(Serialized::defaults(Config::default()));
    if let Some(config_file) = config_file {
        log::info!("config location: {}", config_file.display());
        figment = figment.admerge(Toml::file(config_file));
    } else {
        log::warn!("no config file specified");
    }
    figment = figment.admerge(Env::prefixed("TARDI_"));

    // extract the configuration
    let mut config: Config = figment.extract()?;
    log::debug!("configuration read: {:#?}", config);

    // patch the history_file
    let project_dirs = ProjectDirs::from("", "", "Tardi");
    config.repl.history_file = config.repl.history_file.or_else(|| {
        project_dirs
            .as_ref()
            .map(|pd| pd.data_local_dir().join("repl-history.txt").to_path_buf())
    });

    Ok(config)
}

pub fn init_default_config() -> Result<PathBuf> {
    let config_file = default_config_file().ok_or(Error::MissingConfiguration)?;

    if fs::exists(&config_file)? {
        log::warn!(
            "{} exists. not overwriting with default.",
            config_file.display()
        );
    } else {
        let contents = include_str!("./data/default_config.toml");
        fs::write(&config_file, contents)?;
    }

    Ok(config_file)
}

#[cfg(test)]
mod tests;
