use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::Compiler;
use crate::config::Config;
use crate::env::Environment;
use crate::error::Result;
use crate::module::{KERNEL, SANDBOX};
use crate::shared::{shared, Shared};
use crate::value::lambda::Lambda;
use crate::value::{Value, ValueData};
use crate::vm::VM;

pub trait Execute {
    fn run(&mut self, env: Shared<Environment>, compiler: &mut Compiler) -> Result<()>;
    fn stack(&self) -> Vec<Value>;
    fn execute_macro(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        trigger: &ValueData,
        lambda: &Lambda,
        token_buffer: Shared<Value>,
    ) -> Result<()>;
}

// TODO: make the VM the orchestrator and get rid of this?
// XXX: rationalize how the environment gets passed around
pub struct Tardi {
    pub input: Option<String>,
    pub environment: Shared<Environment>,
    pub compiler: Compiler,
    pub executor: VM,
}

impl Tardi {
    pub fn assemble(environment: Environment, compiler: Compiler, executor: VM) -> Self {
        Tardi {
            input: None,
            environment: shared(environment),
            compiler,
            executor,
        }
    }

    // TODO: add bootstrapping to Config and then depreate this
    pub fn new(bootstrap_dir: Option<PathBuf>) -> Result<Self> {
        let mut tardi = Tardi::default();
        tardi.bootstrap(bootstrap_dir)?;
        Ok(tardi)
    }

    pub fn bootstrap(&mut self, bootstrap_dir: Option<PathBuf>) -> Result<()> {
        if let Some(bootstrap_dir) = bootstrap_dir {
            log::trace!("Tardi::bootstrap {:?}", bootstrap_dir);
            if !bootstrap_dir.exists() {
                return Ok(());
            }
            let mut files = bootstrap_dir
                .read_dir()
                .unwrap()
                .filter_map(|dir_entry| dir_entry.ok())
                .map(|dir_entry| dir_entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "tardi"))
                .collect::<Vec<_>>();
            files.sort();
            for file in files {
                log::debug!("bootstrapping from {:?}", file);
                let input = fs::read_to_string(file)?;
                self.execute_str(&input)?;
            }
        } else {
            log::trace!("Tardi::bootstrap internal modules");
            self.execute_module_str(KERNEL, include_str!("../bootstrap/00-core-macros.tardi"))?;
            self.execute_module_str(KERNEL, include_str!("../bootstrap/01-stack-ops.tardi"))?;
            self.execute_module_str(KERNEL, include_str!("../bootstrap/02-core-ops.tardi"))?;
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.input = None;
    }

    pub fn compile_str(&mut self, module_name: &str, input: &str) -> Result<Shared<Environment>> {
        log::debug!("Tardi::compile_str -- {} : {}", module_name, input);
        self.compiler.compile_internal(
            &mut self.executor,
            self.environment.clone(),
            module_name,
            input,
        )?;
        Ok(self.environment.clone())
    }

    pub fn compile_script(&mut self, path: &Path) -> Result<Shared<Environment>> {
        self.compiler
            .compile_script(&mut self.executor, self.environment.clone(), path)?;
        Ok(self.environment.clone())
    }

    pub fn execute(&mut self) -> Result<()> {
        log::debug!("environment:\n{:?}", self.environment.borrow());
        self.executor
            .run(self.environment.clone(), &mut self.compiler)
    }

    pub fn execute_str(&mut self, input: &str) -> Result<()> {
        log::trace!("Tardi::execute_str");
        self.reset();
        self.compile_str(SANDBOX, input)?;
        self.execute()
    }

    pub fn execute_module_str(&mut self, module: &str, input: &str) -> Result<()> {
        log::trace!("Tardi::execute_module_str");
        self.reset();
        self.compile_str(module, input)?;
        self.execute()
    }

    pub fn execute_file(&mut self, path: &Path) -> Result<()> {
        log::trace!("Tardi::execute_file");
        self.reset();
        self.compile_script(path)?;
        self.execute()
    }

    pub fn stack(&self) -> Vec<Value> {
        self.executor.stack()
    }

    // allowing because this is used in tests
    // TODO: can i move this into the test module?
    #[allow(dead_code)]
    pub(crate) fn execute_ip(&mut self, ip: usize) -> Result<()> {
        let bookmark = self.executor.ip;
        self.executor.ip = ip;
        self.execute()?;
        self.executor.ip = bookmark;
        Ok(())
    }
}

// TODO: add bootstrapping from the default directory to here?
// seems like too much. all of this will be obvious defaults,
// but the bootstrap dir will be more often configured
impl Default for Tardi {
    fn default() -> Tardi {
        let environment = Environment::with_builtins(None);
        let compiler = Compiler::default();
        let executor = VM::new();
        Tardi::assemble(environment, compiler, executor)
    }
}

impl From<&Config> for Tardi {
    fn from(config: &Config) -> Self {
        let environment = Environment::with_builtins(Some(config));
        let compiler = Compiler::default();
        let executor = VM::new();
        Tardi::assemble(environment, compiler, executor)
    }
}

#[cfg(test)]
mod tests;
