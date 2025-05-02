use std::convert::TryInto;
use std::fmt::Debug;
use std::fs;

use log::Level;
use module::Loader;

pub mod module;

use crate::config::Config;
use crate::core::{Compile, Execute, Scan};
use crate::env::Environment;
use crate::error::{CompilerError, Error, Result, ScannerError, VMError};
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Callable, Lambda};
use crate::value::{Value, ValueData, ValueVec};
use crate::vm::OpCode;
use crate::{Scanner, VM};

#[derive(Default)]
struct CompileClosure {
    words: Vec<String>,
    instructions: Vec<usize>,
}

#[derive(Default)]
struct ModuleCompiler {
    scanner: Scanner,
}

// XXX: add a module stack. items of it will include the scanner and anything
// else needed to compile the module.
pub struct Compiler {
    environment: Option<Shared<Environment>>,
    loader: Loader,
    module_stack: Vec<ModuleCompiler>,
    closure_stack: Vec<CompileClosure>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: move this somewhere else
fn hoist_result<T>(input: Vec<Result<T>>) -> Result<Vec<T>> {
    input.into_iter().collect()
}

impl From<&Config> for Compiler {
    fn from(config: &Config) -> Self {
        let loader = Loader::from(config);
        Compiler {
            environment: None,
            loader,
            module_stack: Vec::new(),
            closure_stack: Vec::new(),
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            environment: None,
            loader: Loader::default(),
            module_stack: Vec::new(),
            closure_stack: Vec::new(),
        }
    }

    pub fn current_scanner(&self) -> Option<&Scanner> {
        self.module_stack.last().map(|m| &m.scanner)
    }

    pub fn current_scanner_mut(&mut self) -> Option<&mut Scanner> {
        self.module_stack.last_mut().map(|m| &mut m.scanner)
    }

    fn push_module(&mut self, scanner: Scanner) {
        self.module_stack.push(ModuleCompiler { scanner });
    }

    fn pass1<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        input: &str,
    ) -> Result<Vec<Value>> {
        if log::log_enabled!(log::Level::Trace) {
            if input.len() > 24 {
                log::trace!("Compiler::pass1 {:?}...", &input[..20]);
            } else {
                log::trace!("Compiler::pass1 {:?}", input);
            }
        }
        let mut buffer = Vec::new();

        while let Some(result) = self.scan_value() {
            let value = result?;
            log::trace!("Compiler::pass1 read   {}", value);
            log::trace!("Compiler::pass1 buffer {}", ValueVec(&buffer));
            if value.data == ValueData::Macro {
                let lambda = self.compile_macro(executor, env.clone())?;
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
        log::trace!("Compiler::pass2 {}", ValueVec(&values),);

        for value in values {
            self.compile_value(value)?;
        }

        Ok(())
    }

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
            "clear" => self.compile_op(OpCode::Clear),
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
            "?" => self.compile_op(OpCode::Question),
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
            "<predeclare-function>" => self.compile_op(OpCode::PredeclareFunction),
            "apply" => self.compile_op(OpCode::Apply),
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
        words: &[Value],
    ) -> Result<Lambda> {
        // TODO: can I reuse this function for anything else?
        if log::log_enabled!(Level::Trace) {
            let words = Vec::from(words);
            log::trace!("Compiler::compile_list {}", ValueVec(&words));
        }

        self.start_function();
        let mut buffer = Vec::new();

        for word in words {
            if let Some(lambda) = get_macro(env.clone(), &word.data) {
                // TODO: see todo in `pass1` about benchmarking going back and forth
                // between Shared<Value> and Vec<Value>.
                let accumulator = shared(buffer.into());
                executor.execute_macro(
                    env.clone(),
                    self,
                    &word.data,
                    &lambda,
                    accumulator.clone(),
                )?;
                buffer = unshare_clone(accumulator).try_into()?;
            } else {
                log::trace!("Compiler::compile_list -- pushing {}", word);
                buffer.push(word.clone());
            }
        }

        self.pass2(buffer)?;
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
            // TODO: get the pos for this value from the outer punctuation.
            Ok(Lambda {
                name: None,
                immediate: false,
                defined: true,
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
        let op_table_index = self
            .environment
            .as_ref()
            .and_then(|e| e.borrow().get_op_map().get(word).copied());
        if let Some(op_table_index) = op_table_index {
            self.compile_instruction(op_table_index);
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
    ) -> Result<Lambda> {
        log::trace!("Compiler::compile_macro");
        let trigger = self
            .scan_value()
            .ok_or(ScannerError::UnexpectedEndOfInput)?;
        let trigger = trigger?;
        log::trace!("Compiler::compile_macro trigger {}", trigger);

        let body = shared(Value::new(ValueData::List(vec![])));
        self.scan_object_list(
            executor,
            env,
            ValueData::Word(";".to_string()),
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

    pub fn scan_str(&mut self, input: &str) -> Result<Vec<Result<Value>>> {
        let scanner = Scanner::from_input_string(input);
        Ok(scanner.collect())
    }

    pub fn scan_value(&mut self) -> Option<Result<Value>> {
        self.current_scanner_mut().and_then(|s| s.scan_value())
    }

    pub fn scan_value_list(&mut self, delimiter: &ValueData) -> Result<Vec<Value>> {
        self.current_scanner_mut()
            .ok_or_else(|| ScannerError::NotInitialized.into())
            .and_then(|s| s.scan_value_list(delimiter))
    }

    // TODO: when this is done, can I reimplement `scan` to be
    // `scan_value_list(ValueData::EndOfInput)`?
    pub fn scan_object_list<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        delimiter: ValueData,
        accumulator: Shared<Value>,
    ) -> Result<()> {
        log::trace!("Compiler::scan_object_list {}", delimiter);
        if !accumulator.borrow().is_list() {
            return Err(Error::VMError(VMError::TypeMismatch(format!(
                "Expected a list, but got {}",
                accumulator.borrow()
            ))));
        }

        while let Some(value) = self.scan_value() {
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

    pub fn use_module(&mut self, vm: &mut VM) -> Result<()> {
        if let Some(module_word) = self.scan_value() {
            let module_word = module_word?;
            let module_word = module_word
                .lexeme
                .as_ref()
                .ok_or_else(|| CompilerError::UnsupportedToken(format!("{}", module_word)))?;
            // XXX: get the context from the compiler's stack of modules it's compiling
            let module_file = self
                .loader
                .find(&module_word, None)?
                .ok_or_else(|| CompilerError::ModuleNotFound(module_word.clone()))?;
            let input = fs::read_to_string(&module_file)?;
            // XXX: do I need to share this with the env and everything else?
            let scanner = Scanner::from_module(&module_word, &module_file, &input);
            self.push_module(scanner);

            todo!("Compiler::use_module -- values that know about file and module name");
            todo!("Compiler::use_module -- compile with knowledge of existing namespaces");
            todo!("Compiler::use_module -- compile to namespace");
            todo!("Compiler::use_module -- add to env");

            Ok(())
        } else {
            return Err(ScannerError::UnexpectedEndOfInput.into());
        }
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
        input: &str,
    ) -> Result<()> {
        self.environment = Some(env.clone());
        self.push_module(Scanner::from_input_string(input));
        let intermediate = self.pass1(executor, env, input)?;
        self.pass2(intermediate)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
