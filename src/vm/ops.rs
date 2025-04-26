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
            15 => Ok(OpCode::ToR),
            16 => Ok(OpCode::RFrom),
            17 => Ok(OpCode::RFetch),
            18 => Ok(OpCode::CreateList),
            19 => Ok(OpCode::Append),
            20 => Ok(OpCode::Prepend),
            21 => Ok(OpCode::Concat),
            22 => Ok(OpCode::SplitHead),
            23 => Ok(OpCode::CreateString),
            24 => Ok(OpCode::ToString),
            25 => Ok(OpCode::Utf8ToString),
            26 => Ok(OpCode::StringConcat),
            27 => Ok(OpCode::Apply),
            28 => Ok(OpCode::Return),
            29 => Ok(OpCode::Exit),
            30 => Ok(OpCode::Jump),
            31 => Ok(OpCode::JumpStack),
            32 => Ok(OpCode::Function),
            33 => Ok(OpCode::ScanValue),
            34 => Ok(OpCode::ScanValueList),
            35 => Ok(OpCode::ScanObjectList),
            36 => Ok(OpCode::LitStack),
            37 => Ok(OpCode::Compile),
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
