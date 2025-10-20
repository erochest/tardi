#[derive(Debug)]
pub struct Value {
    /// The text of the value.
    /// TODO: Keeping this may be expensive.
    pub text: String,
    /// The byte position of start of the input token.
    pub index: Option<usize>,
    /// The length of the token in bytes.
    pub length: Option<usize>,
}

impl Value {
    pub fn new(text: String, index: usize, length: usize) -> Self {
        Self {
            text,
            index: Some(index),
            length: Some(length),
        }
    }

    pub fn from_string(text: String) -> Self {
        Self {
            text,
            index: None,
            length: None,
        }
    }
}
