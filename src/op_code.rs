use std::convert::TryFrom;

use crate::error::{Error, Result};

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum OpCode {
    GetConstant = 0,
    Add,
    Sub,
    Mult,
    Div,
    Not,
    Equal,
    Less,
    Greater,
    Jump,
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
            5 => Ok(OpCode::Not),
            6 => Ok(OpCode::Equal),
            7 => Ok(OpCode::Less),
            8 => Ok(OpCode::Greater),
            9 => Ok(OpCode::Jump),
            10 => Ok(OpCode::Return),
            code => Err(Error::InvalidOpCode(code)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() {
        assert_eq!(OpCode::try_from(0).unwrap(), OpCode::GetConstant);
        assert_eq!(OpCode::try_from(1).unwrap(), OpCode::Add);
        assert_eq!(OpCode::try_from(2).unwrap(), OpCode::Sub);
        assert_eq!(OpCode::try_from(3).unwrap(), OpCode::Mult);
        assert_eq!(OpCode::try_from(4).unwrap(), OpCode::Div);
        assert_eq!(OpCode::try_from(5).unwrap(), OpCode::Not);
        assert_eq!(OpCode::try_from(6).unwrap(), OpCode::Equal);
        assert_eq!(OpCode::try_from(7).unwrap(), OpCode::Less);
        assert_eq!(OpCode::try_from(8).unwrap(), OpCode::Greater);
        assert_eq!(OpCode::try_from(9).unwrap(), OpCode::Jump);
        assert_eq!(OpCode::try_from(10).unwrap(), OpCode::Return);
        assert!(matches!(
            OpCode::try_from(177),
            Err(Error::InvalidOpCode(177))
        ));
    }
}
