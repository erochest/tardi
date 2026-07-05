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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplConfig {
    pub edit_mode: EditMode,
    pub history_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum EditMode {
    #[default]
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

impl Default for Config {
    fn default() -> Self {
        let current_dir = env::current_dir().unwrap();
        let module_path = vec![current_dir];
        Config {
            repl: ReplConfig::default(),
            module_path,
        }
    }
}

// impl Config {
//     pub fn try_from<T: Provider>(provider: T) -> Result<Self> {
//         Figment::from(provider).extract().map_err(Error::from)
//     }
//
//     pub fn figment() -> Figment {
//         use figment::providers::Env;
//         Figment::from(Config::default()).merge(Env::prefixed("TARDI_"))
//     }
// }
//
// impl From<Config> for rustyline::Config {
//     fn from(val: Config) -> Self {
//         let mut config = rustyline::Config::builder();
//         config = config.edit_mode(val.repl.edit_mode.into());
//         config.build()
//     }
// }
//
// impl Provider for Config {
//     fn metadata(&self) -> Metadata {
//         Metadata::named("Tardi Library Configuration")
//     }
//
//     fn data(&self) -> std::result::Result<Map<Profile, Dict>, figment::Error> {
//         figment::providers::Serialized::defaults(Config::default()).data()
//     }
//
//     // TODO: make repl and script profiles?
//     fn profile(&self) -> Option<Profile> {
//         None
//     }
// }

#[cfg(test)]
mod tests;
