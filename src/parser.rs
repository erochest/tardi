use crate::error::Result;
use crate::scanner::{scan, Token};

pub fn parse(input: &str) -> Result<Vec<Token>> {
    let tokens = scan(input)?;
    Ok(tokens)
}

#[cfg(test)]
mod tests;
