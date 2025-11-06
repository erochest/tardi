use crate::value::Value;

#[derive(Debug)]
pub enum OpCode {
    Noop,
    CreateEnvironment,
    Literal(Value),
}
