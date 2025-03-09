use std::{convert::TryFrom, result};

use crate::error::{Error, Result};

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum OpCode {
    GetConstant = 0,
    Add,
    Sub,
    Mult,
    Div,
    Modulo,
    Not,
    Equal,
    Less,
    Greater,
    Jump,
    MarkJump,
    CallTardiFn,
    ToCallStack,
    FromCallStack,
    CopyCallStack,
    Drop,
    Swap,
    IP,
    Return,
}

impl TryFrom<u8> for OpCode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(OpCode::GetConstant),
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Sub),
            3 => Ok(OpCode::Mult),
            4 => Ok(OpCode::Div),
            5 => Ok(OpCode::Modulo),
            6 => Ok(OpCode::Not),
            7 => Ok(OpCode::Equal),
            8 => Ok(OpCode::Less),
            9 => Ok(OpCode::Greater),
            10 => Ok(OpCode::Jump),
            11 => Ok(OpCode::MarkJump),
            12 => Ok(OpCode::CallTardiFn),
            13 => Ok(OpCode::ToCallStack),
            14 => Ok(OpCode::FromCallStack),
            15 => Ok(OpCode::CopyCallStack),
            16 => Ok(OpCode::Drop),
            17 => Ok(OpCode::Swap),
            18 => Ok(OpCode::IP),
            19 => Ok(OpCode::Return),
            code => Err(Error::InvalidOpCode(code)),
        }
    }
}

impl TryFrom<&str> for OpCode {
    type Error = Error;

    fn try_from(value: &str) -> result::Result<Self, Self::Error> {
        match value {
            "+" => Ok(OpCode::Add),
            "-" => Ok(OpCode::Sub),
            "*" => Ok(OpCode::Mult),
            "/" => Ok(OpCode::Div),
            "%" => Ok(OpCode::Modulo),
            "!" => Ok(OpCode::Not),
            "==" => Ok(OpCode::Equal),
            "<" => Ok(OpCode::Less),
            ">" => Ok(OpCode::Greater),
            ">r" => Ok(OpCode::ToCallStack),
            "r>" => Ok(OpCode::FromCallStack),
            "r@" => Ok(OpCode::CopyCallStack),
            "drop" => Ok(OpCode::Drop),
            "swap" => Ok(OpCode::Swap),
            "IP" => Ok(OpCode::IP),
            "return" => Ok(OpCode::Return),
            _ => Err(Error::InvalidOpCodeName(value.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_u8() {
        assert_eq!(OpCode::try_from(0).unwrap(), OpCode::GetConstant);
        assert_eq!(OpCode::try_from(1).unwrap(), OpCode::Add);
        assert_eq!(OpCode::try_from(2).unwrap(), OpCode::Sub);
        assert_eq!(OpCode::try_from(3).unwrap(), OpCode::Mult);
        assert_eq!(OpCode::try_from(4).unwrap(), OpCode::Div);
        assert_eq!(OpCode::try_from(5).unwrap(), OpCode::Modulo);
        assert_eq!(OpCode::try_from(6).unwrap(), OpCode::Not);
        assert_eq!(OpCode::try_from(7).unwrap(), OpCode::Equal);
        assert_eq!(OpCode::try_from(8).unwrap(), OpCode::Less);
        assert_eq!(OpCode::try_from(9).unwrap(), OpCode::Greater);
        assert_eq!(OpCode::try_from(10).unwrap(), OpCode::Jump);
        assert_eq!(OpCode::try_from(11).unwrap(), OpCode::MarkJump);
        assert_eq!(OpCode::try_from(12).unwrap(), OpCode::CallTardiFn);
        assert_eq!(OpCode::try_from(13).unwrap(), OpCode::ToCallStack);
        assert_eq!(OpCode::try_from(14).unwrap(), OpCode::FromCallStack);
        assert_eq!(OpCode::try_from(15).unwrap(), OpCode::CopyCallStack);
        assert_eq!(OpCode::try_from(16).unwrap(), OpCode::Drop);
        assert_eq!(OpCode::try_from(17).unwrap(), OpCode::Swap);
        assert_eq!(OpCode::try_from(18).unwrap(), OpCode::IP);
        assert_eq!(OpCode::try_from(19).unwrap(), OpCode::Return);
        assert!(matches!(
            OpCode::try_from(177),
            Err(Error::InvalidOpCode(177))
        ));
    }

    #[test]
    fn test_try_from_str() {
        assert_eq!(OpCode::try_from("+").unwrap(), OpCode::Add);
        assert_eq!(OpCode::try_from("-").unwrap(), OpCode::Sub);
        assert_eq!(OpCode::try_from("*").unwrap(), OpCode::Mult);
        assert_eq!(OpCode::try_from("/").unwrap(), OpCode::Div);
        assert_eq!(OpCode::try_from("%").unwrap(), OpCode::Modulo);
        assert_eq!(OpCode::try_from("!").unwrap(), OpCode::Not);
        assert_eq!(OpCode::try_from("==").unwrap(), OpCode::Equal);
        assert_eq!(OpCode::try_from("<").unwrap(), OpCode::Less);
        assert_eq!(OpCode::try_from(">").unwrap(), OpCode::Greater);
        assert_eq!(OpCode::try_from(">r").unwrap(), OpCode::ToCallStack);
        assert_eq!(OpCode::try_from("r>").unwrap(), OpCode::FromCallStack);
        assert_eq!(OpCode::try_from("r@").unwrap(), OpCode::CopyCallStack);
        assert_eq!(OpCode::try_from("drop").unwrap(), OpCode::Drop);
        assert_eq!(OpCode::try_from("swap").unwrap(), OpCode::Swap);
        assert_eq!(OpCode::try_from("return").unwrap(), OpCode::Return);
        assert_eq!(OpCode::try_from("IP").unwrap(), OpCode::IP);
        assert!(OpCode::try_from("oops").is_err());
    }
}
