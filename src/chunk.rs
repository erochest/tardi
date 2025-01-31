use crate::value::Value;

pub struct Chunk {
    pub constants: Vec<Value>,
    pub code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
} 

#[cfg(test)]
mod tests {
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
}