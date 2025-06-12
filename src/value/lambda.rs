use std::{fmt, ptr};

use crate::error::{Result, VMError};
use crate::{Compiler, VM};

/// Function pointer type for VM operations
pub type OpFn = fn(&mut VM, &mut Compiler) -> Result<()>;

/// Function structure for user-defined functions and lambdas
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
pub struct Lambda {
    // TODO: this needs to include the module as well somehow
    /// Optional name (None for lambdas)
    pub name: Option<String>,

    pub immediate: bool,
    pub defined: bool,

    pub callable: Callable,
}

impl Lambda {
    pub fn new_lambda(words: Vec<String>, ip: usize, length: usize) -> Self {
        let callable = Callable::Compiled {
            words,
            ip,
            length,
            is_loop: false,
        };
        Lambda {
            name: None,
            immediate: false,
            defined: true,
            callable,
        }
    }

    pub fn new_builtin(name: &str, function: OpFn) -> Self {
        let name = Some(name.to_string());
        let callable = Callable::BuiltIn { function };
        Lambda {
            name,
            immediate: false,
            defined: true,
            callable,
        }
    }

    pub fn new_compiled(_name: &str, _words: &[String], _ip: usize) -> Self {
        todo!("Lambda::new_compiled")
    }

    pub fn new_macro(_name: &str, _words: &[String], _ip: usize) -> Self {
        todo!("Lambda::new_macro")
    }

    pub fn new_builtin_macro(name: &str, function: OpFn) -> Self {
        let name = Some(name.to_string());
        let callable = Callable::BuiltIn { function };
        Lambda {
            name,
            immediate: true,
            defined: true,
            callable,
        }
    }

    pub fn new_undefined(name: &str) -> Self {
        let name = Some(name.to_string());
        let callable = Callable::Compiled {
            words: vec![],
            ip: 0,
            length: 0,
            is_loop: false,
        };
        Lambda {
            name,
            immediate: false,
            defined: false,
            callable,
        }
    }

    pub fn call(&self, vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
        log::trace!("calling {}", self.name.as_deref().unwrap_or("<lambda>"));

        if !self.defined {
            let name = self.name.clone().unwrap_or("<lambda>".to_string());
            log::error!("calling predeclared and undefined word {}", name);
            return Err(VMError::InvalidWordCall(name).into());
        }

        self.callable.call(vm, compiler)
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self.callable, Callable::BuiltIn { .. })
    }

    pub fn is_compiled(&self) -> bool {
        matches!(self.callable, Callable::Compiled { .. })
    }

    pub fn get_compiled(&self) -> Option<&Callable> {
        match self.callable {
            Callable::Compiled { .. } => Some(&self.callable),
            _ => None,
        }
    }

    pub fn get_ip(&self) -> Option<usize> {
        match self.callable {
            Callable::Compiled { ip, .. } => Some(ip),
            _ => None,
        }
    }

    pub fn get_length(&self) -> Option<usize> {
        match self.callable {
            Callable::Compiled { length, .. } => Some(length),
            _ => None,
        }
    }

    pub fn is_loop(&self) -> bool {
        match self.callable {
            Callable::Compiled { is_loop, .. } => is_loop,
            _ => false,
        }
    }

    pub fn set_loop(&mut self, new_is_loop: bool) {
        log::trace!("setting a loop {}", new_is_loop);
        if let Callable::Compiled {
            ref mut is_loop, ..
        } = self.callable
        {
            *is_loop = new_is_loop;
        }
    }

    pub fn define_function(&mut self, new_ip: usize, new_length: usize) -> Result<()> {
        log::trace!("Lambda::define_function {:?} => {}", self.name, new_ip);
        if let Callable::Compiled {
            ref mut ip,
            ref mut length,
            ..
        } = self.callable
        {
            self.defined = true;
            *ip = new_ip;
            *length = new_length;
            Ok(())
        } else {
            Err(VMError::TypeMismatch("setting ip of a builtin".to_string()).into())
        }
    }
}

/// Enum representing different types of callable objects
#[derive(Debug, Clone)]
pub enum Callable {
    /// Built-in function implemented in Rust
    BuiltIn { function: OpFn },
    /// User-defined function or lambda
    Compiled {
        words: Vec<String>,
        ip: usize,
        length: usize,
        is_loop: bool,
    },
}

impl Callable {
    fn call(&self, vm: &mut VM, compiler: &mut Compiler) -> Result<()> {
        match self {
            Callable::BuiltIn { function, .. } => {
                log::trace!("calling built-in function");
                function(vm, compiler)
            }
            Callable::Compiled {
                ip: instructions,
                words,
                is_loop,
                ..
            } => {
                // TODO: have this run the IP for macros
                log::trace!("calling compiled function: {:?}", words);
                vm.push_ip(*is_loop)?;
                log::trace!("moving instruction pointer to {}", instructions);
                vm.ip = *instructions;
                Ok(())
            }
        }
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        // Could also compare the words. Would this be better?
        match (self, other) {
            (Callable::BuiltIn { function: a }, Callable::BuiltIn { function: b }) => ptr::eq(a, b),
            (Callable::Compiled { ip: a, .. }, Callable::Compiled { ip: b, .. }) => a == b,
            _ => false,
        }
    }
}

impl std::hash::Hash for Callable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Callable::BuiltIn { function } => {
                std::ptr::hash(function, state);
            }
            Callable::Compiled { ip, .. } => {
                ip.hash(state);
            }
        }
    }
}

impl PartialOrd for Callable {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl fmt::Display for Lambda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "{}", name)
        } else {
            write!(f, "{}", self.callable)
        }
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callable::BuiltIn { .. } => write!(f, "fn"),
            Callable::Compiled { words, ip, .. } => write!(f, "[ {} ]@{}", words.join(" "), ip),
        }
    }
}
