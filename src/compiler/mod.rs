use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::fs;
use std::mem;
use std::path::Path;
use std::result;

use log::Level;
use module::{Module, SANDBOX};

pub mod error;
pub mod module;

use crate::compiler::error::{CompilerError, CompilerResult};
use crate::core::Execute;
use crate::env::Environment;
use crate::error::{Error, Result};
use crate::scanner::error::ScannerError;
use crate::scanner::Source;
use crate::shared::{shared, unshare_clone, Shared};
use crate::value::lambda::{Callable, Lambda};
use crate::value::{Value, ValueData, ValueVec};
use crate::vm::OpCode;
use crate::{Scanner, VM};

#[derive(Default)]
struct LambdaCompiler {
    words: Vec<String>,
    instructions: Vec<usize>,
}

// TODO: is there a faster hashmap I should use here?
#[allow(dead_code)]
#[derive(Default)]
pub struct ModuleCompiler {
    scanner: Scanner,

    /// The name of the module.
    name: String,
}

impl TryFrom<&Module> for ModuleCompiler {
    type Error = CompilerError;

    fn try_from(module: &Module) -> CompilerResult<ModuleCompiler> {
        let source = match module.path {
            Some(ref path) => Source::Module {
                name: module.name.clone(),
                path: path.clone(),
            },
            None => Source::InputString,
        };
        let scanner = Scanner::try_from(source)?;
        Ok(ModuleCompiler {
            scanner,
            name: module.get_key(),
        })
    }
}

#[derive(Default)]
pub struct Compiler {
    environment: Option<Shared<Environment>>,
    module_stack: Vec<ModuleCompiler>,
    lambda_stack: Vec<LambdaCompiler>,
}

impl Compiler {
    pub fn current_module_compiler(&self) -> Option<&ModuleCompiler> {
        self.module_stack.last()
    }

    pub fn current_module_compiler_mut(&mut self) -> Option<&mut ModuleCompiler> {
        self.module_stack.last_mut()
    }

    pub fn current_scanner(&self) -> Option<&Scanner> {
        self.current_module_compiler().map(|m| &m.scanner)
    }

    pub fn current_scanner_mut(&mut self) -> Option<&mut Scanner> {
        self.current_module_compiler_mut().map(|m| &mut m.scanner)
    }

    pub fn get_current_module_mut(&mut self, _env: Shared<Environment>) -> Option<&mut Module> {
        todo!("Compiler::get_current_module_mut")
    }

    fn start_module_compiler(&mut self, name: &str, scanner: Scanner) {
        let name = name.to_string();
        let mc = ModuleCompiler { scanner, name };
        self.module_stack.push(mc);
    }

    fn finish_module_compiler(&mut self) -> Result<()> {
        self.module_stack.pop().ok_or_else(|| {
            CompilerError::InvalidState("no current module being compiled".to_string())
        })?;
        Ok(())
    }

    fn pass1<E: Execute>(&mut self, executor: &mut E) -> Result<Vec<Value>> {
        let mut buffer = Vec::new();
        let module_name = self
            .current_module_compiler()
            .map(|m| m.name.clone())
            .ok_or_else(|| CompilerError::InvalidState("no current module".to_string()))?;

        while let Some(result) = self.scan_value() {
            let value = result?;
            log::trace!("Compiler::pass1 read   {}", value);
            log::trace!("Compiler::pass1 buffer {}", ValueVec(&buffer));
            // TODO: also compile functions here? there'd be fewer constants hanging around.
            if value.data == ValueData::Macro {
                let lambda = self.compile_macro(executor)?;
                self.environment
                    .as_ref()
                    .unwrap()
                    .borrow_mut()
                    .add_macro(&module_name, lambda)?;
            } else if let Some(lambda) = self.get_macro(&value.data) {
                buffer = self.execute_macro(executor, buffer, &value.data, lambda)?;
            } else {
                buffer.push(value);
            }
        }

        Ok(buffer)
    }

    fn pass2(&mut self, values: Vec<Value>) -> CompilerResult<()> {
        log::trace!("Compiler::pass2 {}", ValueVec(&values),);
        for value in values {
            self.compile_value(value)?;
        }
        Ok(())
    }

    fn compile_value(&mut self, value: Value) -> CompilerResult<()> {
        log::trace!("Compiler::compile_value {:?}", value.lexeme);
        // If we're collecting words for a function/lambda, add to the current word list
        if let Some(closure) = self.lambda_stack.last_mut() {
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
            ValueData::Function(ref lambda) if lambda.name.is_none() => {
                self.compile_constant(value)
            }
            ValueData::Function(lambda) => self.add_function(lambda),
            ValueData::Symbol { .. } => self.compile_symbol(value),
            ValueData::Macro => unreachable!("This is handled by the Scanner."),
            // XXX: does this ever get emitted anymore?
            ValueData::EndOfInput => {
                log::trace!("Compiler::compile_value EndOfInput -- emitting Return");
                self.compile_op(OpCode::Return)?;
                Ok(())
            }
            ValueData::Word(_) => unreachable!("ValueData::Word should not be compiled"),
        }
    }

    fn compile_symbol(&mut self, value: Value) -> CompilerResult<()> {
        log::trace!("Compiler::compile_symbol {:?}", value.lexeme);
        let word = value
            .data
            .get_word()
            .ok_or_else(|| CompilerError::UnsupportedToken(format!("{:?}", value)))?;

        match word {
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
            "apply" => self.compile_op(OpCode::Apply),
            "lit" => self.compile_op(OpCode::LitStack),
            "compile" => self.compile_op(OpCode::Compile),
            "stop" => self.compile_op(OpCode::Stop),
            "bye" => self.compile_op(OpCode::Bye),
            _ => self.compile_symbol_call(&value),
        }
    }

    /// Compile a list of words into a lambda.
    pub fn compile_list<E: Execute>(
        &mut self,
        executor: &mut E,
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
            if let Some(lambda) = self.get_macro(&word.data) {
                buffer = self.execute_macro(executor, buffer, &word.data, lambda)?;
            } else {
                log::trace!("Compiler::compile_list -- pushing {}", word);
                buffer.push(word.clone());
            }
        }

        self.pass2(buffer)?;
        self.compile_op(OpCode::Return)?;
        self.end_function().map_err(Error::from)
    }

    fn execute_macro<E: Execute>(
        &mut self,
        executor: &mut E,
        buffer: Vec<Value>,
        word: &ValueData,
        lambda: Shared<Lambda>,
    ) -> Result<Vec<Value>> {
        // TODO: once we get more code to test on, benchmark whether it's better to
        // create `buffer` as a `Value<ValueData::List>` convert it back and forth.
        // It'll depend on how much macros get used.
        log::trace!(
            "Compiler::compile_list executing macro {:?}",
            lambda.borrow().name
        );
        let accumulator = shared(buffer.into());
        executor.execute_macro(
            self.environment.as_ref().unwrap().clone(),
            self,
            word,
            &lambda.borrow().clone(),
            accumulator.clone(),
        )?;
        log::trace!(
            "Compiler::compile_list macro {:?} returned {:#?}",
            word,
            accumulator
        );
        unshare_clone(accumulator).try_into()
    }

    /// Adds an opcode to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn compile_op(&mut self, op: OpCode) -> CompilerResult<()> {
        log::trace!("Compiler::compile_op {:?}", op);
        self.compile_instruction(op.into());
        Ok(())
    }

    /// Adds an opcode and its argument to the current function being defined,
    /// or to the main instruction list if no function is being defined
    pub fn compile_op_arg(&mut self, op: OpCode, arg: usize) -> CompilerResult<()> {
        log::trace!("Compiler::compile_op_arg {:?} {}", op, arg);
        self.compile_op(op)?;
        self.compile_instruction(arg);
        Ok(())
    }

    pub fn compile_instruction(&mut self, arg: usize) {
        if let Some(closure) = self.lambda_stack.last_mut() {
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

    fn compile_constant<T: Into<Value> + Debug>(&mut self, value: T) -> CompilerResult<()> {
        log::trace!("Compiler::compile_constant {:?}", value);
        let const_index = self
            .environment
            .as_ref()
            .map(|e| e.borrow_mut().add_constant(value.into()))
            .ok_or(CompilerError::InvalidOperation(
                "no environment".to_string(),
            ))?;
        self.compile_op_arg(OpCode::Lit, const_index)?;
        Ok(())
    }

    /// Starts a new function definition by pushing a new Vec<usize> onto the function_stack
    pub fn start_function(&mut self) -> usize {
        log::trace!("Compiler::start_function");
        self.lambda_stack.push(LambdaCompiler::default());
        self.lambda_stack.len() - 1
    }

    /// Ends a function definition by popping the top Vec<usize> from function_stack,
    /// appending it to the main instructions, and returning the start index
    pub fn end_function(&mut self) -> CompilerResult<Lambda> {
        let mut lambda = self
            .lambda_stack
            .pop()
            .ok_or_else(|| CompilerError::InvalidOperation("missing begin function".to_string()))?;
        if log::log_enabled!(Level::Trace) {
            log::trace!(
                "Compiler::end_function lambda: [ {} ] {:?}",
                lambda.words.join(" "),
                lambda.instructions
            );
        }

        let jump_target = self
            .environment
            .as_ref()
            .map(|e| e.borrow().instructions_len())
            .unwrap_or_default()
            + 2
            + lambda.instructions.len();
        self.compile_op_arg(OpCode::Jump, jump_target)?;

        // TODO: do more of this instead of cloning.
        let instructions = mem::take(&mut lambda.instructions);
        let ip = self
            .environment
            .as_ref()
            .map(|e| e.borrow_mut().extend_instructions(instructions))
            .unwrap_or_default();

        while lambda.words.last().map(|w| w == "]").unwrap_or(false) {
            lambda.words.pop();
        }

        log::trace!("Compiler::end_function: {} ({:?})", ip, lambda.words);
        // TODO: get the pos for this value from the outer punctuation.
        Ok(Lambda {
            name: None,
            immediate: false,
            defined: true,
            callable: Callable::Compiled {
                words: lambda.words,
                ip,
            },
        })
    }

    /// Compiles a word as a function call
    fn compile_symbol_call(&mut self, value: &Value) -> CompilerResult<()> {
        log::trace!("Compiler::compile_symbol_call {}", value);
        let (module, word) = value.get_symbol().ok_or_else(|| {
            CompilerError::InvalidState(format!(
                "Compiler::compile_symbol_call not a symbol {:?}",
                value
            ))
        })?;
        let op_table_index = self
            .environment
            .as_ref()
            .and_then(|e| e.borrow().get_op_index(module, word));
        log::trace!(
            "Compiler::compile_symbol_call {}::{} => {:?}",
            module,
            word,
            op_table_index
        );
        if let Some(op_table_index) = op_table_index {
            self.compile_instruction(op_table_index);
            Ok(())
        } else {
            self.compile_constant(Value {
                data: ValueData::Symbol {
                    module: module.to_string(),
                    word: word.to_string(),
                },
                ..value.clone()
            })?;
            Ok(())
        }
    }

    /// Compiles a lambda expression
    pub fn compile_lambda(&mut self) -> CompilerResult<()> {
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
    fn add_function(&mut self, lambda: Lambda) -> CompilerResult<()> {
        log::trace!("Compiler::add_function {:?}", lambda.name);
        let module_name = self
            .current_scanner()
            .map(|s| s.source.get_key())
            .ok_or_else(|| CompilerError::InvalidState("unknown path for module".to_string()))?;
        if let Some(env) = self.environment.as_ref() {
            env.borrow_mut()
                .add_to_op_table(&module_name, shared(lambda))?;
            Ok(())
        } else {
            Err(CompilerError::MissingEnvironment)
        }
    }

    fn compile_macro<E: Execute>(&mut self, executor: &mut E) -> Result<Lambda> {
        log::trace!("Compiler::compile_macro");
        let trigger = self.scan_word()?;
        log::trace!("Compiler::compile_macro trigger {}", trigger);

        let body = shared(Value::new(ValueData::List(vec![])));
        self.scan_object_list(executor, ValueData::Word(";".to_string()), body.clone())?;

        log::trace!("Compiler::compile_macro {} => {}", trigger, body.borrow());
        self.start_function();

        let body = unshare_clone(body).try_into()?;
        self.pass2(body)?;
        self.compile_op(OpCode::Stop)?;

        let mut lambda = self.end_function()?;
        lambda.name = trigger.lexeme.clone();
        lambda.immediate = true;

        Ok(lambda)
    }

    pub fn scan_value(&mut self) -> Option<CompilerResult<Value>> {
        self.current_scanner_mut()
            .and_then(|s| s.scan_value().map(|r| r.map_err(CompilerError::from)))
    }

    pub fn scan_word(&mut self) -> Result<Value> {
        let word = self
            .scan_value()
            .ok_or(ScannerError::UnexpectedEndOfInput)?;
        let word = word?;
        Ok(word)
    }

    pub fn scan_value_list(&mut self, delimiter: &ValueData) -> CompilerResult<Vec<Value>> {
        self.current_scanner_mut()
            .ok_or_else(|| ScannerError::NotInitialized.into())
            .and_then(|s| {
                let list = s.scan_value_list(delimiter.clone())?;
                list.into_iter()
                    .collect::<result::Result<Vec<_>, _>>()
                    .map_err(CompilerError::from)
            })
    }

    // TODO: when this is done, can I reimplement `scan` to be
    // `scan_value_list(ValueData::EndOfInput)`?
    pub fn scan_object_list<E: Execute>(
        &mut self,
        executor: &mut E,
        delimiter: ValueData,
        accumulator: Shared<Value>,
    ) -> Result<()> {
        log::trace!("Compiler::scan_object_list {}", delimiter);
        if !accumulator.borrow().is_list() {
            return Err(CompilerError::TypeMismatch(format!(
                "expected a list, but got {}",
                accumulator.borrow()
            ))
            .into());
        }
        // We don't want the module to matter for this, so we'll coerce this
        // back down to a word.
        let delimiter = if let ValueData::Symbol { word, .. } = delimiter {
            ValueData::Word(word.clone())
        } else {
            delimiter
        };

        while let Some(value) = self.scan_value() {
            let value = value?;
            log::trace!("Compiler::scan_object_list ({}) read {}", delimiter, value);
            if value.data == delimiter {
                log::trace!(
                    "Compiler::scan_object_list ({}) returning {}",
                    delimiter,
                    accumulator.borrow()
                );
                return Ok(());
            } else if let Some(lambda) = self.get_macro(&value.data) {
                log::trace!(
                    "Compiler::scan_object_list ({}) executing macro {}: {:?}",
                    delimiter,
                    lambda.borrow(),
                    lambda.borrow()
                );
                executor.execute_macro(
                    self.environment.as_ref().unwrap().clone(),
                    self,
                    &value.data,
                    &lambda.borrow().clone(),
                    accumulator.clone(),
                )?;
                log::trace!(
                    "Compiler::scan_object_list (macro {}) returned {:#?}",
                    delimiter,
                    accumulator
                );
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
        let env = self.environment.as_ref().unwrap().clone();
        let (module_spec, context) = self.find_module_info()?;
        log::trace!("Compiler::use_module {} {:?}", module_spec, context);
        let mut module_spec = module_spec;

        if !env.borrow_mut().handle_internal_module(&module_spec)? {
            let (module_name, module_file) = env
                .borrow()
                .find_module(&module_spec, context)?
                .ok_or_else(|| CompilerError::ModuleNotFound(module_spec.clone()))?;
            log::trace!("Compiler::use_module {} => {:?}", module_name, module_file);
            module_spec = module_name.clone();

            if self.is_being_imported(&module_file) {
                log::trace!("Compiler::use_module {}: import cycle error", module_spec);
                return Err(CompilerError::ImportCycleError(module_name).into());
            }

            if env.borrow().get_module(&module_name).is_some() {
                log::trace!("Compiler::use_module {} already loaded", module_name);
            } else {
                self.create_and_compile_module(vm, &module_name, &module_file)?;
            }
        }

        self.use_into_current_module(&env, &module_spec)?;

        Ok(())
    }

    fn create_and_compile_module(
        &mut self,
        vm: &mut VM,
        module_name: &str,
        module_file: &Path,
    ) -> Result<()> {
        let env = self.environment.as_ref().unwrap().clone();
        let input = fs::read_to_string(module_file)?;
        let scanner = Scanner::from_module(module_name, module_file, &input);
        env.borrow_mut().get_or_create_module_mut(module_name);
        self.compile_module_passes(vm, module_name, scanner)?;
        Ok(())
    }

    fn find_module_info(&mut self) -> Result<(String, Option<&Path>)> {
        let module_word = self.scan_word()?;
        let module_spec = module_word
            .get_word()
            .ok_or_else(|| CompilerError::UnsupportedToken(format!("{}", module_word)))?;
        let context = self
            .current_scanner()
            .ok_or_else(|| CompilerError::InvalidState("missing scanner".to_string()))?
            .source
            .get_path();
        Ok((module_spec.to_string(), context))
    }

    fn compile_module_passes(
        &mut self,
        vm: &mut VM,
        module_name: &str,
        scanner: Scanner,
    ) -> Result<()> {
        self.start_module_compiler(module_name, scanner);
        let intermediate = self.pass1(vm)?;
        self.pass2(intermediate)?;
        self.finish_module_compiler()?;
        Ok(())
    }

    fn use_into_current_module(
        &mut self,
        env: &Shared<Environment>,
        source_name: &str,
    ) -> Result<()> {
        log::trace!("Compiler::use_into_current_module {}", source_name);
        if let Some(Scanner { source, .. }) = self.current_scanner_mut() {
            let key = source.get_key();
            let env = env.clone();
            env.borrow_mut().use_module(source_name, &key)?;
            Ok(())
        } else {
            Err(CompilerError::InvalidState("no scanner".to_string()).into())
        }
    }

    fn get_macro(&self, trigger: &ValueData) -> Option<Shared<Lambda>> {
        log::trace!("Compiler::get_macro {}", trigger);
        if let Some(word) = trigger.get_word() {
            // log::trace!("Compiler::get_macro word {}", word);
            if let Some(module_name) = self.current_module_compiler().as_ref().map(|m| &m.name) {
                // log::trace!("Compiler::get_macro module {}", module_name);
                return self
                    .environment
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .get_op_index(module_name, word)
                    .and_then(|index| {
                        // log::trace!("Compiler::get_macro index {}", index);
                        self.environment
                            .as_ref()
                            .unwrap()
                            .clone()
                            .borrow()
                            .get_op(&0, index)
                            .ok()
                    })
                    .filter(|lambda| {
                        let immediate = lambda.borrow().immediate;
                        // log::trace!("Compiler::get_macro immediate {}", immediate);
                        immediate
                    });
            }
        }
        None
    }

    pub fn compile_repl(
        &mut self,
        vm: &mut VM,
        env: Shared<Environment>,
        input: &str,
    ) -> Result<()> {
        self.compile_scanner(vm, env, Scanner::from_input_string(input))
    }

    pub fn compile_script(
        &mut self,
        vm: &mut VM,
        env: Shared<Environment>,
        file: &Path,
    ) -> Result<()> {
        let input = fs::read_to_string(file)?;
        self.compile_scanner(vm, env, Scanner::from_script(file, &input))
    }

    pub fn compile_module(
        &mut self,
        vm: &mut VM,
        env: Shared<Environment>,
        name: &str,
        file: &Path,
    ) -> Result<()> {
        let input = fs::read_to_string(file)?;
        self.compile_scanner(vm, env, Scanner::from_module(name, file, &input))
    }

    pub fn compile_internal(
        &mut self,
        vm: &mut VM,
        env: Shared<Environment>,
        name: &str,
        input: &str,
    ) -> Result<()> {
        self.compile_scanner(vm, env, Scanner::from_internal_module(name, input))
    }

    pub fn compile_scanner(
        &mut self,
        vm: &mut VM,
        env: Shared<Environment>,
        scanner: Scanner,
    ) -> Result<()> {
        log::trace!("Compiler::compile_scanner {:?}", scanner.source);
        self.environment = Some(env.clone());
        log::trace!(
            "Compiler::compile_scanner -- env set {}",
            self.environment.is_some()
        );
        if let Some(e) = self.environment.as_ref() {
            e.borrow().module_manager.debug_module(SANDBOX);
        }

        let module_name = scanner.source.get_key();
        env.borrow_mut().get_or_create_module_mut(&module_name);
        self.compile_module_passes(vm, &module_name, scanner)?;

        Ok(())
    }

    fn is_being_imported(&self, module_path: &Path) -> bool {
        log::trace!("Compiler::is_being_imported {:?}", module_path);
        for module_scanner in self.module_stack.iter() {
            if module_scanner.scanner.source.get_path().is_some_and(|p| {
                let p = p.canonicalize().unwrap();
                log::trace!("Compiler::is_being_imported checking against {:?}", p);
                p == module_path
            }) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests;
