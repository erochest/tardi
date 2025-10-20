#[derive(Debug)]
pub struct Value {
    pub text: String,
    pub index: Option<usize>,
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
