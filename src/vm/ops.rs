use std::convert::TryFrom;

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Lit,
    Dup,
    Swap,
    Rot,
    Drop,
    StackSize,
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
    ScanToken,
    ScanTokenList,
    ScanValueList,
    Const,
    LitStack,
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
            5 => Ok(OpCode::StackSize),
            6 => Ok(OpCode::Add),
            7 => Ok(OpCode::Subtract),
            8 => Ok(OpCode::Multiply),
            9 => Ok(OpCode::Divide),
            10 => Ok(OpCode::Equal),
            11 => Ok(OpCode::Less),
            12 => Ok(OpCode::Greater),
            13 => Ok(OpCode::Not),
            14 => Ok(OpCode::ToR),
            15 => Ok(OpCode::RFrom),
            16 => Ok(OpCode::RFetch),
            17 => Ok(OpCode::CreateList),
            18 => Ok(OpCode::Append),
            19 => Ok(OpCode::Prepend),
            20 => Ok(OpCode::Concat),
            21 => Ok(OpCode::SplitHead),
            22 => Ok(OpCode::CreateString),
            23 => Ok(OpCode::ToString),
            24 => Ok(OpCode::Utf8ToString),
            25 => Ok(OpCode::StringConcat),
            26 => Ok(OpCode::Call),
            27 => Ok(OpCode::CallStack),
            28 => Ok(OpCode::Return),
            29 => Ok(OpCode::Jump),
            30 => Ok(OpCode::JumpStack),
            31 => Ok(OpCode::Function),
            32 => Ok(OpCode::ScanToken),
            33 => Ok(OpCode::ScanTokenList),
            34 => Ok(OpCode::ScanValueList),
            35 => Ok(OpCode::Const),
            36 => Ok(OpCode::LitStack),
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
