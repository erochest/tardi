use std::convert::TryInto;
use std::fmt::Debug;

use log::Level;

use crate::core::{Compile, Execute, Scan};
use crate::env::Environment;
use crate::error::{CompilerError, Error, Result, ScannerError, VMError};
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Callable, Lambda};
use crate::value::{Value, ValueData, ValueVec};
use crate::vm::OpCode;
use crate::Scanner;

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

    fn pass1<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        input: &str,
    ) -> Result<Vec<Value>> {
        if log::log_enabled!(log::Level::Trace) {
            if input.len() > 24 {
                log::trace!("Compiler::pass1 {:?}...", &input[..20]);
            } else {
                log::trace!("Compiler::pass1 {:?}", input);
            }
        }
        scanner.set_source(input);
        let mut buffer = Vec::new();

        // TODO: this needs to convert to the `TokenType::Dup` style codes
        while let Some(result) = scanner.scan_value() {
            let value = result?;
            log::trace!("Compiler::pass1 read   {}", value);
            log::trace!("Compiler::pass1 buffer {}", ValueVec(&buffer));
            if value.data == ValueData::Macro {
                let lambda = self.compile_macro(executor, env.clone(), scanner)?;
                env.borrow_mut().add_macro(lambda)?;
            } else if let Some(lambda) = get_macro(env.clone(), &value.data) {
                log::trace!("Compiler::pass1 executing macro {:?}", lambda.name);
                // TODO: once we get more code to test on, benchmark whether it's better to
                // create `buffer` as a `Value<ValueData::List>` convert it back and forth.
                // It'll depend on how much macros get used.
                let accumulator = shared(buffer.into());
                executor.execute_macro(
                    env.clone(),
                    self,
                    scanner,
                    &value.data,
                    &lambda,
                    accumulator.clone(),
                )?;
                buffer = unshare_clone(accumulator).try_into()?;
            } else {
                buffer.push(value);
            }
        }

        Ok(buffer)
    }

    fn pass2(&mut self, values: Vec<Value>) -> Result<()> {
        if log::log_enabled!(log::Level::Trace) {
            log::trace!(
                "Compiler::pass2 {:?}",
                values
                    .iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            );
        }
        for value in values {
            log::trace!("Compile::pass2 {}", value);
            self.compile_value(value)?;
        }
        Ok(())
    }

    // TODO: I Suspect that this isn't handling the `}` correctly before
    // `scan-value-list`. (hint: it needs to be a literal.)
    fn compile_value(&mut self, value: Value) -> Result<()> {
        log::trace!("Compiler::compile_value {:?}", value.lexeme);
        // If we're collecting words for a function/lambda, add to the current word list
        if let Some(closure) = self.closure_stack.last_mut() {
            if let Some(ref lexeme) = value.lexeme {
                closure.words.push(lexeme.clone());
            }
        }
        match value.data {
            ValueData::Integer(_)
            | ValueData::Float(_)
            | ValueData::Boolean(_)
            | ValueData::Char(_)
            | ValueData::List(_)
            | ValueData::String(_)
            | ValueData::Address(_)
            | ValueData::Literal(_) => self.compile_constant(value),
            ValueData::Function(ref lambda) => {
                if lambda.name.is_none() {
                    self.compile_constant(value)
                } else {
                    self.add_function(lambda)
                }
            }
            ValueData::Word(_) => self.compile_word(value),
            ValueData::Macro => unreachable!("This is handled by the Scanner."),
            ValueData::EndOfInput => {
                log::trace!("Compiler::compile_value EndOfInput -- emitting Return");
                self.compile_op(OpCode::Return)?;
                Ok(())
            }
        }
    }

    fn compile_word(&mut self, value: Value) -> Result<()> {
        log::trace!("Compiler::compile_word {:?}", value.lexeme);
        let word = if let ValueData::Word(w) = value.data {
            Ok(w)
        } else {
            Err(CompilerError::UnsupportedToken(format!("{:?}", value)))
        }?;

        match word.as_str() {
            "dup" => self.compile_op(OpCode::Dup),
            "swap" => self.compile_op(OpCode::Swap),
            "rot" => self.compile_op(OpCode::Rot),
            "drop" => self.compile_op(OpCode::Drop),
            "stack-size" => self.compile_op(OpCode::StackSize),
            ">r" => self.compile_op(OpCode::ToR),
            "r>" => self.compile_op(OpCode::RFrom),
            "r@" => self.compile_op(OpCode::RFetch),
            "+" => self.compile_op(OpCode::Add),
            "-" => self.compile_op(OpCode::Subtract),
            "*" => self.compile_op(OpCode::Multiply),
            "/" => self.compile_op(OpCode::Divide),
            "==" => self.compile_op(OpCode::Equal),
            "!=" => {
                self.compile_op(OpCode::Equal)?;
                self.compile_op(OpCode::Not)
            }
            "<" => self.compile_op(OpCode::Less),
            ">" => self.compile_op(OpCode::Greater),
            "<=" => {
                self.compile_op(OpCode::Greater)?;
                self.compile_op(OpCode::Not)
            }
            ">=" => {
                self.compile_op(OpCode::Less)?;
                self.compile_op(OpCode::Not)
            }
            "!" => self.compile_op(OpCode::Not),
            "<list>" => self.compile_op(OpCode::CreateList),
            "append" => self.compile_op(OpCode::Append),
            "prepend" => self.compile_op(OpCode::Prepend),
            "concat" => self.compile_op(OpCode::Concat),
            "split-head!" => self.compile_op(OpCode::SplitHead),
            "<string>" => self.compile_op(OpCode::CreateString),
            ">string" => self.compile_op(OpCode::ToString),
            "utf8>string" => self.compile_op(OpCode::Utf8ToString),
            "string-concat" => self.compile_op(OpCode::StringConcat),
            "<function>" => self.compile_op(OpCode::Function),
            "call" => self.compile_op(OpCode::CallStack),
            "{" => {
                // Start a new function compilation
                self.start_function();
                Ok(())
            }
            "}" => {
                // End the current function/lambda
                if !self.closure_stack.is_empty() {
                    self.compile_lambda()
                } else {
                    Err(Error::CompilerError(CompilerError::UnmatchedBrace))
                }
            }
            "lit" => self.compile_op(OpCode::LitStack),
            "scan-value" => self.compile_op(OpCode::ScanValue),
            "scan-value-list" => self.compile_op(OpCode::ScanValueList),
            "scan-object-list" => self.compile_op(OpCode::ScanObjectList),
            "compile" => self.compile_op(OpCode::Compile),
            "exit" => self.compile_op(OpCode::Exit),
            _ => self.compile_word_call(&word),
        }
    }

    /// Compile a list of words into a lambda.
    pub fn compile_list<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        words: &[Value],
    ) -> Result<Lambda> {
        // TODO: can I reuse this function for anything else?
        self.start_function();
        let mut buffer = Vec::new();

        for word in words {
            if let Some(lambda) = get_macro(env.clone(), &word.data) {
                // TODO: see todo in `pass1`
                let accumulator = shared(buffer.into());
                executor.execute_macro(
                    env.clone(),
                    self,
                    scanner,
                    &word.data,
                    &lambda,
                    accumulator.clone(),
                )?;
                buffer = unshare_clone(accumulator).try_into()?;
            } else {
                buffer.push(word.clone());
            }
        }

        self.pass2(buffer)?;
        log::trace!("Compiler::compile_list -- emitting Return");
        self.compile_op(OpCode::Return)?;
        self.end_function()
    }

    /// Adds an opcode to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn compile_op(&mut self, op: OpCode) -> Result<()> {
        log::trace!("Compiler::compile_op {:?}", op);
        self.compile_instruction(op.into());
        Ok(())
    }

    /// Adds an opcode and its argument to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn compile_op_arg(&mut self, op: OpCode, arg: usize) -> Result<()> {
        log::trace!("Compiler::compile_op_arg {:?} {}", op, arg);
        self.compile_op(op)?;
        self.compile_instruction(arg);
        Ok(())
    }

    pub fn compile_instruction(&mut self, arg: usize) {
        if let Some(closure) = self.closure_stack.last_mut() {
            log::trace!(
                "Compiler::compile_instruction {} @ {} of closure",
                arg,
                closure.instructions.len()
            );
            closure.instructions.push(arg);
        } else if let Some(e) = self.environment.as_ref() {
            log::trace!("Compiler::compile_instruction {}", arg);
            e.borrow_mut().add_instruction(arg)
        }
    }

    fn compile_constant<T: Into<Value> + Debug>(&mut self, value: T) -> Result<()> {
        log::trace!("Compiler::compile_constant {:?}", value);
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
        log::trace!("Compiler::start_function");
        self.closure_stack.push(CompileClosure::default());
        self.closure_stack.len() - 1
    }

    /// Ends a function definition by popping the top Vec<usize> from function_stack,
    /// appending it to the main instructions, and returning the start index
    pub fn end_function(&mut self) -> Result<Lambda> {
        if let Some(mut closure) = self.closure_stack.pop() {
            if log::log_enabled!(Level::Trace) {
                log::trace!(
                    "Compiler::end_function closure: [ {} ] {:?}",
                    closure.words.join(" "),
                    closure.instructions
                );
            }
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
            while closure.words.last().map(|w| w == "}").unwrap_or(false) {
                closure.words.pop();
            }
            log::trace!("Compiler::end_function: {} ({:?})", ip, closure.words);
            // TODO: get the pos for this value
            Ok(Lambda {
                name: None,
                immediate: false,
                callable: Callable::Compiled {
                    words: closure.words,
                    ip,
                },
            })
        } else {
            Err(CompilerError::InvalidOperation("missing begin function".to_string()).into())
        }
    }

    /// Compiles a word as a function call
    fn compile_word_call(&mut self, word: &str) -> Result<()> {
        log::trace!("Compiler::compile_word_call {:?}", word);
        if let Some(op_table_index) = self
            .environment
            .as_ref()
            .and_then(|e| e.borrow().get_op_map().get(word).copied())
        {
            // Right now this only handles non-recursive calls.
            // TODO: to handle recursive calls, if the function doesn't
            // have a valid address (say zero), then put the function's
            // index on the stack and use CallStack.
            // TODO: is there a way to do this without the `Call`?
            // Can I just offset the IP by the number of build-ins
            // and find the function's index that way?
            self.compile_op_arg(OpCode::Call, op_table_index)?;
            Ok(())
        } else {
            self.compile_constant(Value::with_lexeme(ValueData::Word(word.to_string()), word))?;
            Ok(())
        }
    }

    /// Compiles a function definition
    fn compile_function(&mut self) -> Result<()> {
        // The Function opcode expects a name string and a lambda on the stack
        log::trace!("Compiler::compile_function");
        self.compile_op(OpCode::Function)?;
        Ok(())
    }

    /// Compiles a lambda expression
    pub fn compile_lambda(&mut self) -> Result<()> {
        log::trace!("Compiler::compile_lambda -- emitting Return");
        self.compile_op(OpCode::Return)?;

        let lambda = self.end_function()?;
        let value = Value::new(ValueData::Function(lambda));
        let const_index = self
            .environment
            .as_ref()
            .map(|e| e.borrow_mut().add_constant(value))
            .unwrap_or_default();

        self.compile_op_arg(OpCode::Lit, const_index)?;

        Ok(())
    }

    /// Adds a function defined in a macro to the environment
    fn add_function(&mut self, lambda: &Lambda) -> Result<()> {
        log::trace!("Compiler::add_function {:?}", lambda.name);
        if let Some(env) = self.environment.as_ref() {
            env.borrow_mut().add_to_op_table(shared(lambda.clone()));
            Ok(())
        } else {
            Err(Error::CompilerError(CompilerError::MissingEnvironment))
        }
    }

    fn compile_macro<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
    ) -> Result<Lambda> {
        log::trace!("Compiler::compile_macro");
        let trigger = scanner
            .scan_value()
            .ok_or(ScannerError::UnexpectedEndOfInput)?;
        let trigger = trigger?;
        log::trace!("Compiler::compile_macro trigger {}", trigger);

        let body = shared(Value::new(ValueData::List(vec![])));
        self.scan_object_list(
            executor,
            env,
            ValueData::Word(";".to_string()),
            scanner,
            body.clone(),
        )?;
        log::trace!("Compiler::compile_macro {} => {}", trigger, body.borrow());
        self.start_function();
        let body = unshare_clone(body).try_into()?;
        self.pass2(body)?;
        self.compile_op(OpCode::Exit)?;
        let mut lambda = self.end_function()?;
        lambda.name = trigger.lexeme.clone();
        lambda.immediate = true;

        Ok(lambda)
    }

    // TODO: when this is done, can I reimplement `scan` to be
    // `scan_value_list(ValueData::EndOfInput)`?
    pub fn scan_object_list<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        delimiter: ValueData,
        scanner: &mut Scanner,
        accumulator: Shared<Value>,
    ) -> Result<()> {
        log::trace!("Compiler::scan_object_list {}", delimiter);
        if !accumulator.borrow().is_list() {
            return Err(Error::VMError(VMError::TypeMismatch(format!(
                "Expected a list, but got {}",
                accumulator.borrow()
            ))));
        }

        while let Some(value) = scanner.scan_value() {
            let value = value?;
            log::trace!("Compiler::scan_object_list ({}) read {}", delimiter, value);
            if value.data == delimiter {
                if log::log_enabled!(Level::Trace) {
                    log::trace!(
                        "Compiler::scan_object_list ({}) returning {}",
                        delimiter,
                        accumulator.borrow()
                    );
                }
                return Ok(());
            } else if let Some(lambda) = get_macro(env.clone(), &value.data) {
                log::trace!(
                    "Compiler::scan_object_list ({}) executing macro {}",
                    delimiter,
                    lambda
                );
                executor.execute_macro(
                    env.clone(),
                    self,
                    scanner,
                    &value.data,
                    &lambda,
                    accumulator.clone(),
                )?;
            } else {
                accumulator
                    .borrow_mut()
                    .get_list_mut()
                    .unwrap()
                    .push(shared(value));
            }
        }

        Err(ScannerError::UnexpectedEndOfInput.into())
    }
}

fn get_macro(env: Shared<Environment>, trigger: &ValueData) -> Option<Lambda> {
    env.borrow().get_macro(trigger).cloned()
}

impl Compile for Compiler {
    fn compile<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        input: &str,
    ) -> Result<()> {
        self.environment = Some(env.clone());
        let intermediate = self.pass1(executor, env, scanner, input)?;
        self.pass2(intermediate)?;
        Ok(())
    }

    // TODO: where is this getting called? still needed?
    fn compile_lambda<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        input: &str,
    ) -> Result<()> {
        let index = self.start_function();
        self.compile(executor, env, scanner, input)?;
        // TODO: probably need to clean up the function we in-process.
        // I'm not fixing it now because I need to do that everywhere.
        let function = self.end_function()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
