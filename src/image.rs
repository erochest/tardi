use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::path::Path;

use crate::env::Environment;
use crate::error::{EnvironmentError, Result};
use crate::value::lambda::{Callable, Lambda, OpFn};
use crate::value::Value;

#[derive(Serialize, Deserialize)]
pub struct ImageFormat {
    pub version: String,
    pub builtins: Vec<String>,
    pub env: SerializedEnvironment,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedEnvironment {
    pub constants: Vec<Value>,
    pub instructions: Vec<usize>,
    pub op_table: Vec<SerializedLambda>,
    pub op_map: HashMap<String, usize>,
    pub macro_table: HashMap<String, SerializedLambda>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedLambda {
    name: Option<String>,
    immediate: bool,
    defined: bool,
    callable: SerializedCallable,
}

#[derive(Serialize, Deserialize)]
pub enum SerializedCallable {
    BuiltIn,
    Compiled { words: Vec<String>, ip: usize },
}

impl From<Lambda> for SerializedLambda {
    fn from(lambda: Lambda) -> Self {
        SerializedLambda {
            name: lambda.name,
            immediate: lambda.immediate,
            defined: lambda.defined,
            callable: lambda.callable.into(),
        }
    }
}

impl From<Callable> for SerializedCallable {
    fn from(callable: Callable) -> Self {
        match callable {
            Callable::BuiltIn { .. } => SerializedCallable::BuiltIn,
            Callable::Compiled { words, ip } => SerializedCallable::Compiled { words, ip },
        }
    }
}

impl SerializedLambda {
    pub fn into_lambda(self, builtins: &HashMap<String, OpFn>) -> Result<Lambda> {
        let callable = match self.callable {
            SerializedCallable::BuiltIn => {
                // Name must exist for builtins
                let name = self
                    .name
                    .as_ref()
                    .ok_or_else(|| EnvironmentError::MissingBuiltinName)?;
                let function = builtins
                    .get(name)
                    .ok_or_else(|| EnvironmentError::MissingBuiltin(name.clone()))?;
                Callable::BuiltIn {
                    function: *function,
                }
            }
            SerializedCallable::Compiled { words, ip } => Callable::Compiled { words, ip },
        };

        Ok(Lambda {
            name: self.name.clone(),
            immediate: self.immediate,
            defined: self.defined,
            callable,
        })
    }
}

impl From<Environment> for SerializedEnvironment {
    fn from(env: Environment) -> Self {
        SerializedEnvironment {
            constants: env.constants,
            instructions: env.instructions,
            op_table: env
                .op_table
                .into_iter()
                .map(|lambda| (*lambda.borrow()).clone().into())
                .collect(),
            op_map: env.op_map,
            macro_table: env
                .macro_table
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl ImageFormat {
    pub fn new(env: Environment, builtins: Vec<String>) -> Self {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let env = SerializedEnvironment::from(env);
        ImageFormat {
            version,
            builtins,
            env,
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let file = std::fs::File::create(path)?;
        ciborium::ser::into_writer(self, file)
            .map_err(|err| EnvironmentError::SerializationError(format!("{:?}", err)))?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let image: ImageFormat = ciborium::de::from_reader(file)
            .map_err(|err| EnvironmentError::DeserializationError(format!("{:?}", err)))?;
        Ok(image)
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn verify_builtins(&self, current_builtins: &[String]) -> Result<()> {
        if self.builtins != current_builtins {
            return Err(EnvironmentError::BuiltinMismatch {
                expected: self.builtins.clone(),
                found: current_builtins.to_vec(),
            }
            .into());
        }
        Ok(())
    }
}
