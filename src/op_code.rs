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
}

impl TryFrom<u8> for OpCode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(OpCode::GetConstant),
            1 => Ok(OpCode::Add),
            2 => Ok(OpCode::Sub),
            3 => Ok(OpCode::Mult),
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
        assert!(matches!(OpCode::try_from(177), Err(Error::InvalidOpCode(177))));
    }
}
