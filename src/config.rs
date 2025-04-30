use std::convert::TryFrom;

use figment::value::{Dict, Map};
use figment::{Figment, Metadata, Profile, Provider};
use rustyline::ConditionalEventHandler;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    edit_mode: EditMode,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        Config {
            edit_mode: EditMode::Emacs,
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
        config = config.edit_mode(self.edit_mode.into());
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
