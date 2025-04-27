use std::{fmt, ptr};

use crate::error::{Result, VMError};
use crate::{Compiler, Scanner, VM};

/// Function pointer type for VM operations
pub type OpFn = fn(&mut VM, &mut Compiler, &mut Scanner) -> Result<()>;

/// Function structure for user-defined functions and lambdas
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Lambda {
    /// Optional name (None for lambdas)
    pub name: Option<String>,

    pub immediate: bool,
    pub defined: bool,

    pub callable: Callable,
}

impl Lambda {
    pub fn new_lambda(words: Vec<String>, ip: usize) -> Self {
        let callable = Callable::Compiled { words, ip };
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

    pub fn new_compiled(name: &str, words: &[String], ip: usize) -> Self {
        todo!("Lambda::new_compiled")
    }

    pub fn new_macro(name: &str, words: &[String], ip: usize) -> Self {
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
        };
        Lambda {
            name,
            immediate: true,
            defined: false,
            callable,
        }
    }

    pub fn call(&self, vm: &mut VM, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
        log::trace!("calling {}", self.name.as_deref().unwrap_or("<lambda>"));

        if !self.defined {
            let name = self.name.clone().unwrap_or("<lambda>".to_string());
            log::error!("calling predeclared and undefined word {}", name);
            return Err(VMError::InvalidWordCall(name).into());
        }

        self.callable.call(vm, compiler, scanner)
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

    pub fn define_function(&mut self, ip: usize) -> Result<()> {
        log::trace!("Lambda::define_function {:?} => {}", self.name, ip);
        if let Callable::Compiled { ref words, .. } = self.callable {
            self.defined = true;
            // TODO: yuck. is there some better way to set this field?
            self.callable = Callable::Compiled {
                ip,
                words: words.clone(),
            };
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
    Compiled { words: Vec<String>, ip: usize },
}

impl Callable {
    fn call(&self, vm: &mut VM, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
        match self {
            Callable::BuiltIn { function, .. } => {
                log::trace!("calling built-in function");
                function(vm, compiler, scanner)
            }
            Callable::Compiled {
                ip: instructions,
                words,
            } => {
                log::trace!("calling compiled function: {:?}", words);
                vm.push_ip()?;
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
            Callable::Compiled { words, ip } => write!(f, "{{ {} }} @ {}", words.join(" "), ip),
        }
    }
}
