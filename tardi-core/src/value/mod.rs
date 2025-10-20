#[derive(Debug)]
pub struct Value {
    pub text: String,
    pub index: Option<usize>,
}

impl Value {
    pub fn new(text: String, index: usize) -> Self {
        Self {
            text,
            index: Some(index),
        }
    }

    pub fn from_string(text: String) -> Self {
        Self { text, index: None }
    }
}
