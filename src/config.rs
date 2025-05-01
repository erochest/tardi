use std::convert::TryFrom;
use std::path::PathBuf;

use directories::ProjectDirs;
use figment::value::{Dict, Map};
use figment::{Figment, Metadata, Profile, Provider};
use rustyline::ConditionalEventHandler;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repl: ReplConfig,
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

impl Into<rustyline::EditMode> for EditMode {
    fn into(self) -> rustyline::EditMode {
        match self {
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
        Config {
            repl: ReplConfig::default(),
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

impl Into<rustyline::Config> for Config {
    fn into(self) -> rustyline::Config {
        let mut config = rustyline::Config::builder();
        config = config.edit_mode(self.repl.edit_mode.into());
        config.build()
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Tardi Configuration")
    }

    fn data(&self) -> std::result::Result<Map<Profile, Dict>, figment::Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}
