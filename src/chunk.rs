use std::fmt;

use ahash::{HashMap, HashMapExt};

use crate::error::{Error, Result};
use crate::op_code::OpCode;
use crate::value::{Function, Value};
use crate::vm::{Return, VM};

pub struct TardiFn {
    pub name: String,
    pub function: Box<dyn FnMut(&mut VM) -> Result<()>>,
}

impl TardiFn {
    pub fn new(name: &str, function: Box<dyn FnMut(&mut VM) -> Result<()>>) -> Self {
        TardiFn {
            name: name.to_string(),
            function,
        }
    }

    pub fn call(&mut self, vm: &mut VM) -> Result<()> {
        (*self.function)(vm)
    }
}

impl fmt::Debug for TardiFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<TardiFn>")
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    pub constants: Vec<Value>,
    pub code: Vec<u8>,
    pub builtins: Vec<TardiFn>,
    pub builtin_index: HashMap<String, usize>,
    pub dictionary: HashMap<String, Function>,
}

// TODO: debugging output of a chunk
impl Chunk {
    pub fn new() -> Self {
        let (builtins, builtin_index) = define_builtins();
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            builtins,
            builtin_index,
            dictionary: HashMap::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn push_op_code(&mut self, op_code: OpCode, param: u8) {
        self.code.push(op_code as u8);
        self.code.push(param);
    }
}

fn define_builtins() -> (Vec<TardiFn>, HashMap<String, usize>) {
    let mut builtins = Vec::new();
    let mut index = HashMap::new();

    let name = "call".to_string();
    let tardi_fn = TardiFn {
        name: name.clone(),
        function: Box::new(|vm: &mut VM| {
            let top = vm.stack.pop().ok_or(Error::StackUnderflow)?;
            if let Value::Lambda(_, ip) = top {
                vm.call_stack.push(Return::new(vm.ip + 1));
                vm.ip = ip - 1;
            } else {
                return Err(Error::UncallableObject(top));
            }
            Ok(())
        }),
    };
    index.insert(name.clone(), builtins.len());
    builtins.push(tardi_fn);

    (builtins, index)
}

#[cfg(test)]
mod tests {
    use crate::op_code::OpCode;

    use super::*;

    #[test]
    fn test_add_constant() {
        let mut chunk = Chunk::new();
        let constant = Value::Integer(10);
        let index = chunk.add_constant(constant.clone());
        assert_eq!(index, 0);
        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], constant);
    }

    #[test]
    fn test_push_opcode() {
        let mut chunk = Chunk::new();
        let constant = Value::Integer(10);
        let index = chunk.add_constant(constant.clone());

        chunk.push_op_code(OpCode::GetConstant, index as u8);

        assert_eq!(chunk.code.len(), 2);
        assert_eq!(chunk.code, vec![0, 0]);
    }
}
