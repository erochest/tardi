use crate::op_code::OpCode;

#[derive(Default)]
pub struct Module {
    pub instructions: Vec<OpCode>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }
}
