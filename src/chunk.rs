use ahash::{HashMap, HashMapExt};

use crate::op_code::OpCode;
use crate::value::{Function, Value};

#[derive(Debug, Default)]
pub struct Chunk {
    pub constants: Vec<Value>,
    pub code: Vec<u8>,
    pub dictionary: HashMap<String, Function>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
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
