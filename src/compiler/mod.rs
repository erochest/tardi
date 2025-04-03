use crate::error::{CompilerError, Error, Result};
use crate::scanner::{Token, TokenType};
use crate::vm::value::{Callable, Function, Shared, Value};
use crate::vm::OpCode;
use crate::Environment;

use super::Compile;

#[derive(Default)]
struct CompileClosure {
    words: Vec<String>,
    instructions: Vec<usize>,
}

pub struct Compiler {
    environment: Option<Shared<Environment>>,
    closure_stack: Vec<CompileClosure>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            environment: None,
            closure_stack: Vec::new(),
        }
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
            TokenType::StackSize => self.compile_op(OpCode::StackSize),
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
                self.start_function();
                Ok(())
            }
            TokenType::RightCurly => {
                // End the current function/lambda
                if !self.closure_stack.is_empty() {
                    self.compile_lambda()
                } else {
                    Err(Error::CompilerError(CompilerError::UnmatchedBrace))
                }
            }
            TokenType::Word(word) => {
                // If we're collecting words for a function/lambda, add to the current word list
                if let Some(closure) = self.closure_stack.last_mut() {
                    closure.words.push(word.clone());
                }
                self.compile_word_call(&word)?;
                Ok(())
            }
            TokenType::Lambda => todo!(),
            TokenType::Error => todo!(),
            TokenType::Eof => todo!(),
        }
    }

    /// Adds an opcode to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn compile_op(&mut self, op: OpCode) -> Result<()> {
        if let Some(closure) = self.closure_stack.last_mut() {
            closure.instructions.push(op.into());
        } else {
            self.environment
                .as_ref()
                .map(|e| e.borrow_mut().add_instruction(op.into()));
        }
        Ok(())
    }

    /// Adds an opcode and its argument to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn compile_op_arg(&mut self, op: OpCode, arg: usize) -> Result<()> {
        self.compile_op(op)?;
        self.compile_instruction(arg);
        Ok(())
    }

    pub fn compile_instruction(&mut self, arg: usize) {
        if let Some(closure) = self.closure_stack.last_mut() {
            closure.instructions.push(arg);
        } else {
            self.environment
                .as_ref()
                .map(|e| e.borrow_mut().add_instruction(arg));
        }
    }

    fn compile_constant<T: Into<Value>>(&mut self, value: T) -> Result<()> {
        let const_index = self
            .environment
            .as_ref()
            .map(|e| e.borrow_mut().add_constant(value.into()))
            .ok_or(Error::CompilerError(CompilerError::InvalidOperation(
                "no environment".to_string(),
            )))?;
        self.compile_op_arg(OpCode::Lit, const_index)?;
        Ok(())
    }

    /// Starts a new function definition by pushing a new Vec<usize> onto the function_stack
    pub fn start_function(&mut self) -> usize {
        self.closure_stack.push(CompileClosure::default());
        self.closure_stack.len() - 1
    }

    /// Ends a function definition by popping the top Vec<usize> from function_stack,
    /// appending it to the main instructions, and returning the start index
    pub fn end_function(&mut self) -> Result<Function> {
        if let Some(closure) = self.closure_stack.pop() {
            let jump_target = self
                .environment
                .as_ref()
                .map(|e| e.borrow().instructions_len())
                .unwrap_or_default()
                + 2
                + closure.instructions.len();
            self.compile_op_arg(OpCode::Jump, jump_target)?;
            let ip = self
                .environment
                .as_ref()
                .map(|e| {
                    e.borrow_mut()
                        .extend_instructions(closure.instructions.clone())
                })
                .unwrap_or_default();
            Ok(Function {
                name: None,
                words: closure.words,
                ip,
            })
        } else {
            // If there's no function being defined, return current instruction pointer
            // TODO: Should this be an error?
            Ok(Function {
                name: None,
                words: vec![],
                ip: self
                    .environment
                    .as_ref()
                    .map(|e| e.borrow().instructions_len())
                    .unwrap_or_default(),
            })
        }
    }

    /// Compiles a word as a function call
    fn compile_word_call(&mut self, word: &str) -> Result<()> {
        if let Some(index) = self
            .environment
            .as_ref()
            .map(|e| e.borrow().get_op_map().get(word).copied())
            .unwrap_or_default()
        {
            // Right now this only handles non-recursive calls.
            // TODO: to handle recursive calls, if the function doesn't
            // have a valid address (say zero), then put the function's
            // index on the stack and use CallStack.
            self.compile_op_arg(OpCode::Call, index)?;
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
        self.compile_op(OpCode::Function)?;
        Ok(())
    }

    /// Compiles a lambda expression
    fn compile_lambda(&mut self) -> Result<()> {
        self.compile_op(OpCode::Return)?;

        let function = self.end_function()?;
        let callable = Callable::Fn(function);
        let const_index = self
            .environment
            .as_ref()
            .map(|e| e.borrow_mut().add_constant(Value::Function(callable)))
            .unwrap_or_default();

        self.compile_op_arg(OpCode::Lit, const_index)?;

        Ok(())
    }
}

impl Compile for Compiler {
    fn compile(&mut self, env: Shared<Environment>, tokens: Vec<Result<Token>>) -> Result<()> {
        self.environment = Some(env);
        let tokens: Result<Vec<Token>> = tokens.into_iter().collect();
        for token in tokens? {
            self.compile_token(token)?;
        }
        self.compile_op(OpCode::Return)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
