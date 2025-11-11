use crate::op_code::OpCode;

#[derive(Debug, Default)]
pub struct Module {
    pub name: String,
    pub instructions: Vec<OpCode>,
    // TODO: constants
    // TODO: list of functions
    // TODO: name look-up
    // TODO: stack
    // Am I conflating modules with the VM environment?
}

impl Module {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        Self {
            name,
            instructions: Vec::new(),
        }
    }
}
