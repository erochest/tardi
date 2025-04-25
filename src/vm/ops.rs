use std::convert::TryFrom;

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Lit,
    Dup,
    Swap,
    // TODO: move `rot` to bootstrapping
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
    Apply,  // Call a function object on the stack
    Return, // Return from a function
    Exit,   // Shortcut execution at the end of macros
    Jump,   // Jump to a specific instruction
    // TODO: There's nothing using this. Add a word for it
    JumpStack, // Jump to an instruction from the stack
    Function,
    ScanValue,
    ScanValueList,
    ScanObjectList,
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
            26 => Ok(OpCode::Apply),
            27 => Ok(OpCode::Return),
            28 => Ok(OpCode::Exit),
            29 => Ok(OpCode::Jump),
            30 => Ok(OpCode::JumpStack),
            31 => Ok(OpCode::Function),
            32 => Ok(OpCode::ScanValue),
            33 => Ok(OpCode::ScanValueList),
            34 => Ok(OpCode::ScanObjectList),
            35 => Ok(OpCode::LitStack),
            36 => Ok(OpCode::Compile),
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
