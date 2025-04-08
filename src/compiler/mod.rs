use crate::error::{CompilerError, Error, Result};
use crate::scanner::{Token, TokenType};
use crate::shared::{shared, Shared};
use crate::value::{Callable, Function, SharedValue, Value};
use crate::vm::OpCode;
use crate::{Environment, Scan};

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

// TODO: move this somewhere else
fn hoist_result<T>(input: Vec<Result<T>>) -> Result<Vec<T>> {
    Ok(input
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .collect())
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            environment: None,
            closure_stack: Vec::new(),
        }
    }

    fn pass1<S: Scan>(&mut self, scanner: &mut S, input: &str) -> Result<Vec<Value>> {
        // TODO: needs to populate scanning functions in environment
        // TODO: needs to check if they're already there first though
        let tokens = scanner.scan(input)?;
        let tokens = hoist_result(tokens)?;
        let tokens_len = tokens.len();
        let mut i = 0;
        let mut buffer = Vec::new();

        while i < tokens_len {
            if let Some(token) = tokens.get(i) {
                i += 1;
                if let TokenType::MacroStart = token.token_type {
                    i = self.compile_macro(&mut buffer, i, &tokens);
                } else {
                    buffer.push(Value::Token(token.clone()));
                }
            }
        }

        Ok(buffer)
    }

    fn pass2(&mut self, values: Vec<Value>) -> Result<()> {
        for value in values {
            self.compile_value(value)?;
        }
        Ok(())
    }

    fn compile_value(&mut self, value: Value) -> Result<()> {
        match value {
            Value::Integer(_)
            | Value::Float(_)
            | Value::Boolean(_)
            | Value::Char(_)
            | Value::List(_)
            | Value::String(_)
            | Value::Address(_) => self.compile_constant(value),
            Value::Function(ref callable) => {
                if callable.is_lambda() {
                    self.compile_constant(value)
                } else {
                    self.add_function(callable.clone())
                }
            }
            Value::Token(token) => self.compile_token(token),
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
            TokenType::MacroStart => unimplemented!("this gets handled by the scanner"),
            TokenType::Lambda => todo!(),
            TokenType::Error => todo!(),
            TokenType::EndOfInput => {
                self.compile_op(OpCode::Return)?;
                Ok(())
            }
        }
    }

    /// Adds an opcode to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn compile_op(&mut self, op: OpCode) -> Result<()> {
        if let Some(closure) = self.closure_stack.last_mut() {
            closure.instructions.push(op.into());
        } else if let Some(e) = self.environment.as_ref() {
            e.borrow_mut().add_instruction(op.into())
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
        } else if let Some(e) = self.environment.as_ref() {
            e.borrow_mut().add_instruction(arg)
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

    /// Adds a function defined in a macro to the environment
    fn add_function(&mut self, callable: Callable) -> Result<()> {
        if let Some(env) = self.environment.as_ref() {
            env.borrow_mut().add_to_op_table(shared(callable));
            Ok(())
        } else {
            Err(Error::CompilerError(CompilerError::MissingEnvironment))
        }
    }

    // if let Ok(token) = result {
    //     if let TokenType::MacroStart = token.token_type {
    //         // TODO: use scan_token
    //         let trigger = self.scan_token()?;
    //         // TODO: use scan_value_list instead here
    //         let body = self.scan_token_list(&TokenType::Word(";".to_string()))?;
    //         // TODO: compile the body
    //         // TODO: get the compiled body's ip
    //         let function = Callable::Fn(Function {
    //             name: Some(trigger.to_string()),
    //             words: body.iter().map(|v| v.to_string()).collect(),
    //             ip: todo!(),
    //         });
    //     }
    // TODO: check in the env if this is a macro trigger
    // TODO: load VM and env to execute the macro
    // TODO: call the macro
    // TODO: get the macro results
    fn compile_macro(&self, buffer: &mut Vec<Value>, i: usize, tokens: &[Token]) -> usize {
        todo!()
    }
}

impl Compile for Compiler {
    fn compile<S: Scan>(
        &mut self,
        env: Shared<Environment>,
        scanner: &mut S,
        input: &str,
    ) -> Result<()> {
        self.environment = Some(env);
        let intermediate = self.pass1(scanner, input)?;
        self.pass2(intermediate)?;
        Ok(())
    }

    fn compile_lambda<S: Scan>(
        &mut self,
        env: Shared<Environment>,
        scanner: &mut S,
        input: &str,
    ) -> Result<()> {
        let index = self.start_function();
        self.compile(env, scanner, input)?;
        // TODO: probably need to clean up the function we in-process.
        // I'm not fixing it now because I need to do that everywhere.
        let function = self.end_function()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
