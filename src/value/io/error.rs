use std::{error, fmt};

use crate::error::Error;

// TODO: pull this into its own module
#[derive(Debug)]
pub enum TardiIoError {
    ResourceClosed(String),
}

impl fmt::Display for TardiIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TardiIoError::ResourceClosed(name) => write!(f, "resource closed: {}", name),
        }
    }
}

impl error::Error for TardiIoError {}

impl From<TardiIoError> for Error {
    fn from(value: TardiIoError) -> Self {
        Error::TardiError(Box::new(value))
    }
}
