use std::convert::TryFrom;

use crate::error::Error;

// TODO: /mod (divmod)
// TODO: move things out of opcodes and just have native functions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Lit,
    Dup,
    Swap,
    // TODO: move `rot` to bootstrapping
    Rot,
    Drop,
    Clear,
    StackSize,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Less,
    Greater,
    Not,
    Question,
    ToR,
    RFrom,
    RFetch,
    // Function-related operations
    Apply,  // Call a function object on the stack
    Return, // Return from a function
    Stop,   // Shortcut execution at the end of macros
    Bye,    // Exit everything
    // TODO: do i need versions of this able to index larger numbers in the op table?
    Jump, // Jump to a specific instruction
    // TODO: There's nothing using this. Add a word for it
    JumpStack, // Jump to an instruction from the stack
    LitStack,
    Compile,
}

impl From<OpCode> for usize {
    fn from(op: OpCode) -> Self {
        op as usize
    }
}

impl TryFrom<usize> for OpCode {
    type Error = Error;

    fn try_from(value: usize) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Lit),
            1 => Ok(OpCode::Dup),
            2 => Ok(OpCode::Swap),
            3 => Ok(OpCode::Rot),
            4 => Ok(OpCode::Drop),
            5 => Ok(OpCode::Clear),
            6 => Ok(OpCode::StackSize),
            7 => Ok(OpCode::Add),
            8 => Ok(OpCode::Subtract),
            9 => Ok(OpCode::Multiply),
            10 => Ok(OpCode::Divide),
            11 => Ok(OpCode::Equal),
            12 => Ok(OpCode::Less),
            13 => Ok(OpCode::Greater),
            14 => Ok(OpCode::Not),
            15 => Ok(OpCode::Question),
            16 => Ok(OpCode::ToR),
            17 => Ok(OpCode::RFrom),
            18 => Ok(OpCode::RFetch),
            19 => Ok(OpCode::Apply),
            20 => Ok(OpCode::Return),
            21 => Ok(OpCode::Stop),
            22 => Ok(OpCode::Bye),
            23 => Ok(OpCode::Jump),
            24 => Ok(OpCode::JumpStack),
            25 => Ok(OpCode::LitStack),
            26 => Ok(OpCode::Compile),
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
