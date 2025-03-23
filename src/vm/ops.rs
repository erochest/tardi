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
    NotEqual,
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
            10 => Ok(OpCode::NotEqual),
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
            _ => Err(Error::InvalidOpCode(value)),
        }
    }
}
