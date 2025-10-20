#[derive(Debug)]
pub struct Value {
    pub text: String,
}

impl Value {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}
