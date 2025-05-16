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
    Stop,   // Shortcut execution at the end of macros
    Bye,    // Exit everything
    // TODO: do i need versions of this able to index larger numbers in the op table?
    Jump, // Jump to a specific instruction
    // TODO: There's nothing using this. Add a word for it
    JumpStack, // Jump to an instruction from the stack
    Function,
    PredeclareFunction,
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
            15 => Ok(OpCode::Question),
            16 => Ok(OpCode::ToR),
            17 => Ok(OpCode::RFrom),
            18 => Ok(OpCode::RFetch),
            19 => Ok(OpCode::CreateList),
            20 => Ok(OpCode::Append),
            21 => Ok(OpCode::Prepend),
            22 => Ok(OpCode::Concat),
            23 => Ok(OpCode::SplitHead),
            24 => Ok(OpCode::CreateString),
            25 => Ok(OpCode::ToString),
            26 => Ok(OpCode::Utf8ToString),
            27 => Ok(OpCode::StringConcat),
            28 => Ok(OpCode::Apply),
            29 => Ok(OpCode::Return),
            30 => Ok(OpCode::Stop),
            31 => Ok(OpCode::Bye),
            32 => Ok(OpCode::Jump),
            33 => Ok(OpCode::JumpStack),
            34 => Ok(OpCode::Function),
            35 => Ok(OpCode::PredeclareFunction),
            36 => Ok(OpCode::ScanValue),
            37 => Ok(OpCode::ScanValueList),
            38 => Ok(OpCode::ScanObjectList),
            39 => Ok(OpCode::LitStack),
            40 => Ok(OpCode::Compile),
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
