mod program;

pub use self::program::Program;
use crate::error::{CompilerError, Error, Result};
use crate::scanner::{Token, TokenType};
use crate::vm::value::{shared, Callable, Function, Value};
use crate::vm::{create_op_table, OpCode};

pub struct Compiler {
    program: Program,
    /// Stack of words being collected for the current function/lambda
    word_stack: Vec<Vec<String>>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            program: Program::with_builtins(),
            word_stack: Vec::new(),
        }
    }

    pub fn compile(&mut self, tokens: impl Iterator<Item = Result<Token>>) -> Result<Program> {
        for token_result in tokens {
            let token = token_result?;
            self.compile_token(token)?;
        }
        self.compile_op(OpCode::Return)?;
        Ok(self.program.clone())
    }

    fn compile_token(&mut self, token: Token) -> Result<()> {
        match token.token_type {
            TokenType::Integer(value) => self.compile_constant(value),
            TokenType::Float(value) => self.compile_constant(value),
            TokenType::Boolean(value) => self.compile_constant(value),
            TokenType::Char(value) => self.compile_constant(value),
            TokenType::Dup => self.compile_op(OpCode::Dup),
            TokenType::Swap => self.compile_op(OpCode::Swap),
            TokenType::Rot => self.compile_op(OpCode::Rot),
            TokenType::Drop => self.compile_op(OpCode::Drop),
            TokenType::ToR => self.compile_op(OpCode::ToR),
            TokenType::RFrom => self.compile_op(OpCode::RFrom),
            TokenType::RFetch => self.compile_op(OpCode::RFetch),
            TokenType::Plus => self.compile_op(OpCode::Add),
            TokenType::Dash => self.compile_op(OpCode::Subtract),
            TokenType::Star => self.compile_op(OpCode::Multiply),
            TokenType::Slash => self.compile_op(OpCode::Divide),
            TokenType::EqualEqual => self.compile_op(OpCode::Equal),
            TokenType::BangEqual => {
                self.compile_op(OpCode::Equal)?;
                self.compile_op(OpCode::Not)
            }
            TokenType::Less => self.compile_op(OpCode::Less),
            TokenType::Greater => self.compile_op(OpCode::Greater),
            TokenType::LessEqual => {
                self.compile_op(OpCode::Greater)?;
                self.compile_op(OpCode::Not)
            }
            TokenType::GreaterEqual => {
                self.compile_op(OpCode::Less)?;
                self.compile_op(OpCode::Not)
            }
            TokenType::Bang => self.compile_op(OpCode::Not),
            TokenType::CreateList => self.compile_op(OpCode::CreateList),
            TokenType::Append => self.compile_op(OpCode::Append),
            TokenType::Prepend => self.compile_op(OpCode::Prepend),
            TokenType::Concat => self.compile_op(OpCode::Concat),
            TokenType::SplitHead => self.compile_op(OpCode::SplitHead),
            TokenType::String(value) => self.compile_constant(value),
            TokenType::CreateString => self.compile_op(OpCode::CreateString),
            TokenType::ToString => self.compile_op(OpCode::ToString),
            TokenType::Utf8ToString => self.compile_op(OpCode::Utf8ToString),
            TokenType::StringConcat => self.compile_op(OpCode::StringConcat),
            TokenType::Function => self.compile_op(OpCode::Function),
            TokenType::Call => self.compile_op(OpCode::CallStack),
            TokenType::LeftCurly => {
                // Start a new function compilation
                self.program.start_function();
                // Start collecting words for a new function/lambda
                self.word_stack.push(Vec::new());
                Ok(())
            }
            TokenType::RightCurly => {
                // End the current function/lambda
                if let Some(words) = self.word_stack.pop() {
                    self.compile_lambda(words)
                } else {
                    Err(Error::CompilerError(CompilerError::UnmatchedBrace))
                }
            }
            TokenType::Word(word) => {
                // If we're collecting words for a function/lambda, add to the current word list
                if let Some(words) = self.word_stack.last_mut() {
                    words.push(word);
                    Ok(())
                } else {
                    // Otherwise, treat as a function call
                    self.compile_word_call(&word)
                }
            }
            _ => Err(Error::CompilerError(CompilerError::UnsupportedToken(
                format!("{:?}", token),
            ))),
        }
    }

    fn compile_constant<T: Into<Value>>(&mut self, value: T) -> Result<()> {
        let const_index = self.program.add_constant(value.into());
        self.program.add_op_arg(OpCode::Lit, const_index);
        Ok(())
    }

    fn compile_op(&mut self, op: OpCode) -> Result<()> {
        self.program.add_op(op);
        Ok(())
    }

    /// Compiles a word as a function call
    fn compile_word_call(&mut self, word: &str) -> Result<()> {
        if let Some(&index) = self.program.get_op_map().get(word) {
            // Right now this only handles non-recursive calls.
            // TODO: to handle recursive calls, if the function doesn't
            // have a valid address (say zero), then put the function's
            // index on the stack and use CallStack.
            self.program.add_op_arg(OpCode::Call, index);
            Ok(())
        } else {
            Err(Error::CompilerError(CompilerError::UndefinedWord(
                word.to_string(),
            )))
        }
    }

    /// Compiles a function definition
    fn compile_function(&mut self) -> Result<()> {
        // The Function opcode expects a name string and a lambda on the stack
        self.program.add_op(OpCode::Function);
        Ok(())
    }

    /// Compiles a lambda expression
    fn compile_lambda(&mut self, words: Vec<String>) -> Result<()> {
        // Add return instruction
        self.program.add_op(OpCode::Return);

        // End the function and get its start address
        let start_addr = self.program.end_function();

        // Create the Function object
        let function = Function {
            name: None,
            words: words.clone(),
            instructions: start_addr,
        };

        // Create a callable and add it to constants
        let callable = Callable::Fn(function);
        let const_index = self.program.add_constant(Value::Function(shared(callable)));

        // Emit instruction to load the function
        self.program.add_op_arg(OpCode::Lit, const_index);

        Ok(())
    }
}

#[cfg(test)]
mod tests;
