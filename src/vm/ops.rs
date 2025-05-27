use std::convert::TryFrom;

use crate::error::Error;

// TODO: /mod (divmod)
// TODO: move things out of opcodes and just have native functions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Nop,
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
    Break,
    Continue,
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
            0 => Ok(OpCode::Nop),
            1 => Ok(OpCode::Lit),
            2 => Ok(OpCode::Dup),
            3 => Ok(OpCode::Swap),
            4 => Ok(OpCode::Rot),
            5 => Ok(OpCode::Drop),
            6 => Ok(OpCode::Clear),
            7 => Ok(OpCode::StackSize),
            8 => Ok(OpCode::Add),
            9 => Ok(OpCode::Subtract),
            10 => Ok(OpCode::Multiply),
            11 => Ok(OpCode::Divide),
            12 => Ok(OpCode::Equal),
            13 => Ok(OpCode::Less),
            14 => Ok(OpCode::Greater),
            15 => Ok(OpCode::Not),
            16 => Ok(OpCode::Question),
            17 => Ok(OpCode::ToR),
            18 => Ok(OpCode::RFrom),
            19 => Ok(OpCode::RFetch),
            20 => Ok(OpCode::Apply),
            21 => Ok(OpCode::Return),
            22 => Ok(OpCode::Stop),
            23 => Ok(OpCode::Bye),
            24 => Ok(OpCode::Jump),
            25 => Ok(OpCode::JumpStack),
            26 => Ok(OpCode::LitStack),
            27 => Ok(OpCode::Compile),
            28 => Ok(OpCode::Break),
            29 => Ok(OpCode::Continue),
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
