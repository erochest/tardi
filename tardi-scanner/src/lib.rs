use tardi_core::value::Value;

#[derive(Debug)]
pub struct Scanner {}

impl Scanner {
    pub fn from_string(_input: &str) -> Self {
        Self {}
    }
}

impl Iterator for Scanner {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[cfg(test)]
mod tests;
