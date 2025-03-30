use std::convert::TryFrom;

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Lit,
    Dup,
    Swap,
    Rot,
    Drop,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Less,
    Greater,
    Not,
    ToR,
    RFrom,
    RFetch,
    CreateList,
    Append,
    Prepend,
    Concat,
    SplitHead,
    CreateString,
    ToString,
    Utf8ToString,
    StringConcat,
    // Function-related operations
    Call,      // Call a function by its index in the op_table
    CallStack, // Call a function from the stack
    Return,    // Return from a function
    Jump,      // Jump to a specific instruction
    // TODO: There's nothing using this. Add a word for it
    JumpStack, // Jump to an instruction from the stack
    Function,
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
            5 => Ok(OpCode::Add),
            6 => Ok(OpCode::Subtract),
            7 => Ok(OpCode::Multiply),
            8 => Ok(OpCode::Divide),
            9 => Ok(OpCode::Equal),
            10 => Ok(OpCode::Less),
            11 => Ok(OpCode::Greater),
            12 => Ok(OpCode::Not),
            13 => Ok(OpCode::ToR),
            14 => Ok(OpCode::RFrom),
            15 => Ok(OpCode::RFetch),
            16 => Ok(OpCode::CreateList),
            17 => Ok(OpCode::Append),
            18 => Ok(OpCode::Prepend),
            19 => Ok(OpCode::Concat),
            20 => Ok(OpCode::SplitHead),
            21 => Ok(OpCode::CreateString),
            22 => Ok(OpCode::ToString),
            23 => Ok(OpCode::Utf8ToString),
            24 => Ok(OpCode::StringConcat),
            25 => Ok(OpCode::Call),
            26 => Ok(OpCode::CallStack),
            27 => Ok(OpCode::Return),
            28 => Ok(OpCode::Jump),
            29 => Ok(OpCode::JumpStack),
            30 => Ok(OpCode::Function),
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
