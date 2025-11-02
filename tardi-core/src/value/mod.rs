#[derive(Debug)]
pub enum ValueData {
    // Booleans
    Bool(bool),

    // signed integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),

    // unsigned integers
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),

    // words
    Word(String),

    // strings
    String(String),
}

#[derive(Debug)]
pub struct Value {
    /// The semantic value/data of the Value
    pub data: ValueData,
    /// The text of the value.
    /// TODO: Keeping this may be expensive.
    pub text: String,
    /// The byte position of start of the input token.
    pub index: Option<usize>,
    /// The length of the token in bytes.
    pub length: Option<usize>,
}

impl Value {
    pub fn new(data: ValueData, text: String, index: usize, length: usize) -> Self {
        Self {
            data,
            text,
            index: Some(index),
            length: Some(length),
        }
    }

    pub fn from_string(data: ValueData, text: String) -> Self {
        Self {
            data,
            text,
            index: None,
            length: None,
        }
    }
}
