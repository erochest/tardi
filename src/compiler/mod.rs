use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::path::Path;
use std::{fs, mem, result};

use log::Level;
use module::Loader;

pub mod error;
pub mod module;

use crate::compiler::error::{CompilerError, CompilerResult};
use crate::config::Config;
use crate::core::Execute;
use crate::env::{Environment, Module};
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

    /// This maps a word name to its index in the environment's `op_table`. Macros
    /// have the `immediate` flag set.
    defined: HashMap<String, usize>,

    /// This maps the imported word names to their indexes in the environment's `op_table`.
    /// Macros have the `immediate` flag set.
    imported: HashMap<String, usize>,
}

impl From<ModuleCompiler> for Module {
    fn from(module_compiler: ModuleCompiler) -> Self {
        let (name, path) = match module_compiler.scanner.source {
            Source::InputString => ("sandbox".to_string(), None),
            Source::ScriptFile { path } => (
                path.file_stem().unwrap().to_string_lossy().to_string(),
                Some(path),
            ),
            Source::Module { name, path } => (name, Some(path)),
        };
        Module {
            path,
            name,
            defined: module_compiler.defined,
            imported: module_compiler.imported,
        }
    }
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
            defined: module.defined.clone(),
            imported: module.imported.clone(),
        })
    }
}

impl ModuleCompiler {
    pub fn import_module(&mut self, module: &Module) {
        for (name, index) in module.defined.iter() {
            self.imported.insert(name.clone(), *index);
        }
    }

    pub fn get(&self, word: &str) -> Option<usize> {
        self.defined
            .get(word)
            .or_else(|| self.imported.get(word))
            .copied()
    }

    /// This assumes that the ModuleCompiler also already has everything
    /// defined in the Module, so everything in there can can be
    /// clobbered.
    pub fn merge_into(self, module: &mut Module) {
        log::trace!("ModuleCompiler::merge_into defined {:#?}", self.defined);
        log::trace!(
            "ModuleCompiler::merge_into {} => {}",
            self.name,
            module.name
        );
        module.defined = self.defined;
        module.imported = self.imported;
    }
}

#[derive(Default)]
pub struct Compiler {
    environment: Option<Shared<Environment>>,
    loader: Loader,
    module_stack: Vec<ModuleCompiler>,
    lambda_stack: Vec<LambdaCompiler>,
}

impl From<&Config> for Compiler {
    fn from(config: &Config) -> Self {
        let loader = Loader::from(config);
        Compiler {
            environment: None,
            loader,
            module_stack: Vec::new(),
            lambda_stack: Vec::new(),
        }
    }
}

impl Compiler {
    pub fn current_module(&self) -> Option<&ModuleCompiler> {
        self.module_stack.last()
    }

    pub fn current_module_mut(&mut self) -> Option<&mut ModuleCompiler> {
        self.module_stack.last_mut()
    }

    pub fn current_scanner(&self) -> Option<&Scanner> {
        self.current_module().map(|m| &m.scanner)
    }

    pub fn current_scanner_mut(&mut self) -> Option<&mut Scanner> {
        self.current_module_mut().map(|m| &mut m.scanner)
    }

    fn start_module_compiler(&mut self, module: &Module, scanner: Scanner) {
        // XXX: need to load the kernel words into `imported`
        // XXX: need to be able to handle if it's scanning more
        // for an existing, "finished" source, like repl input.
        let mc = ModuleCompiler {
            scanner,
            name: module.get_key(),
            defined: module.defined.clone(),
            imported: module.imported.clone(),
        };
        self.module_stack.push(mc);
    }

    fn finish_module_compiler(&mut self, module_stub: &mut Module) -> Result<()> {
        let mc = self.module_stack.pop().ok_or_else(|| {
            CompilerError::InvalidState("no current module being compiled".to_string())
        })?;
        // TODO: things are getting defined directly in the module in the enviroment. can i just
        // not do module compilers?
        // mc.merge_into(module_stub);
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn get_current_module_def(&self, word: &str) -> Option<usize> {
        self.current_module()
            .and_then(|m| m.defined.get(word))
            .copied()
    }

    fn pass1<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
    ) -> Result<Vec<Value>> {
        let mut buffer = Vec::new();
        let module_name = self
            .current_module()
            .map(|m| m.name.clone())
            .ok_or_else(|| CompilerError::InvalidState("no current module".to_string()))?;

        while let Some(result) = self.scan_value() {
            let value = result?;
            log::trace!("Compiler::pass1 read   {}", value);
            log::trace!("Compiler::pass1 buffer {}", ValueVec(&buffer));
            // TODO: also compile functions here? there'd be fewer constants hanging around.
            if value.data == ValueData::Macro {
                let lambda = self.compile_macro(executor, env.clone())?;
                env.borrow_mut().add_macro(&module_name, lambda)?;
            } else if let Some(lambda) = self.get_macro(env.clone(), &value.data) {
                log::trace!("Compiler::pass1 executing macro {:?}", lambda.borrow().name);
                // TODO: once we get more code to test on, benchmark whether it's better to
                // create `buffer` as a `Value<ValueData::List>` convert it back and forth.
                // It'll depend on how much macros get used.
                let accumulator = shared(buffer.into());
                executor.execute_macro(
                    env.clone(),
                    self,
                    &value.data,
                    &lambda.borrow().clone(),
                    accumulator.clone(),
                )?;
                buffer = unshare_clone(accumulator).try_into()?;
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
        log::trace!("Compiler::compile_word {:?}", value.lexeme);
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
            "<function>" => self.compile_op(OpCode::Function),
            "<predeclare-function>" => self.compile_op(OpCode::PredeclareFunction),
            "apply" => self.compile_op(OpCode::Apply),
            "lit" => self.compile_op(OpCode::LitStack),
            "scan-value" => self.compile_op(OpCode::ScanValue),
            "scan-value-list" => self.compile_op(OpCode::ScanValueList),
            "scan-object-list" => self.compile_op(OpCode::ScanObjectList),
            "compile" => self.compile_op(OpCode::Compile),
            "exit" => self.compile_op(OpCode::Exit),
            _ => self.compile_symbol_call(&value),
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
            // TODO: some of this seems duplicated from `pass1`. DRY it up
            if let Some(lambda) = self.get_macro(env.clone(), &word.data) {
                log::trace!(
                    "Compiler::compile_list executing macro {:?}",
                    lambda.borrow().name
                );
                // TODO: see todo in `pass1` about benchmarking going back and forth
                // between Shared<Value> and Vec<Value>.
                let accumulator = shared(buffer.into());
                executor.execute_macro(
                    env.clone(),
                    self,
                    &word.data,
                    &lambda.borrow().clone(),
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
        self.end_function().map_err(Error::from)
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
            .and_then(|e| e.borrow().get_op_ip(module, word));
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

    pub fn scan_value(&mut self) -> Option<CompilerResult<Value>> {
        self.current_scanner_mut()
            .and_then(|s| s.scan_value().map(|r| r.map_err(CompilerError::from)))
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
        env: Shared<Environment>,
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
            } else if let Some(lambda) = self.get_macro(env.clone(), &value.data) {
                log::trace!(
                    "Compiler::scan_object_list ({}) executing macro {}",
                    delimiter,
                    lambda.borrow()
                );
                executor.execute_macro(
                    env.clone(),
                    self,
                    &value.data,
                    &lambda.borrow().clone(),
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
        // TODO: pull this pattern out into `scan_word`
        let module_word = self
            .scan_value()
            .ok_or(ScannerError::UnexpectedEndOfInput)?;
        let module_word = module_word?;
        let module_spec = module_word
            .get_word()
            .ok_or_else(|| CompilerError::UnsupportedToken(format!("{}", module_word)))?;
        let context = self
            .current_scanner()
            .ok_or_else(|| CompilerError::InvalidState("missing scanner".to_string()))?
            .source
            .get_path();
        let (module_name, module_file) = self
            .loader
            .find(module_spec, context)?
            .ok_or_else(|| CompilerError::ModuleNotFound(module_spec.to_string()))?;

        let env = vm.environment.as_ref().unwrap().clone();
        if let Some(module) = env.borrow().get_module(&module_name) {
            let current_module = self.module_stack.last_mut().ok_or_else(|| {
                CompilerError::InvalidState("no current module being compiled".to_string())
            })?;
            current_module.import_module(module);
            return Ok(());
        }

        // XXX: make sure there aren't loops.
        let input = fs::read_to_string(&module_file)?;
        let scanner = Scanner::from_module(&module_name, &module_file, &input);
        let mut stub = env.borrow().create_module(&module_name);

        self.start_module_compiler(&stub, scanner);

        let intermediate = self.pass1(vm, env.clone())?;
        self.pass2(intermediate)?;

        self.finish_module_compiler(&mut stub)?;

        let current_module = self.module_stack.last_mut().ok_or_else(|| {
            CompilerError::InvalidState("no current module being compiled".to_string())
        })?;
        current_module.import_module(&stub);
        env.borrow_mut().add_module(stub);

        Ok(())
    }

    fn get_macro(&self, env: Shared<Environment>, trigger: &ValueData) -> Option<Shared<Lambda>> {
        // if log::log_enabled!(Level::Trace) {
        //     log::trace!("Compiler::get_macro {}", trigger);
        //     if let Some(word) = trigger.get_word() {
        //         log::trace!("Compiler::get_macro word {}", word);
        //         if let Some(module) = self.current_module() {
        //             log::trace!("Compiler::get_macro module {:?}", module.name);
        //             if let Some(index) = module.get(word) {
        //                 log::trace!("Compiler::get_macro index {}", index);
        //                 if let Some(lambda) = env.borrow().op_table.get(index).cloned() {
        //                     log::trace!("Compiler::get_macro lambda {}", lambda.borrow());
        //                     if lambda.borrow().immediate {
        //                         log::trace!("Compiler::get_macro found macro {}", lambda.borrow());
        //                         return Some(lambda);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     log::trace!("Compiler::get_macro None");
        //     None
        // } else {
        trigger
            .get_word()
            .and_then(|word| self.current_module().and_then(|m| m.get(word)))
            .and_then(|index| env.borrow().op_table.get(index).cloned())
            .filter(|lambda| lambda.borrow().immediate)
        // }
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

    // XXX: make this either take a Source or add a specific method for each
    // type of input.
    pub fn compile_scanner(
        &mut self,
        vm: &mut VM,
        env: Shared<Environment>,
        scanner: Scanner,
    ) -> Result<()> {
        self.environment = Some(env.clone());
        // TODO: this pattern (also in use_module) can be factored out:
        // - make scanner
        // - push_module
        // - pass1
        // - pass2
        // - pop_module
        // - make module
        // - module to env
        let module_name = scanner.source.get_key();
        // TODO: make these methods
        {
            let mut env_borrow = env.borrow_mut();
            let module = env_borrow.get_or_create_module_mut(&module_name);
            log::trace!(
                "Compiler::compile_scanner executing module {:?}",
                module.name
            );
            log::trace!("Compiler::compile_scanner module\n{:?}", module);
            self.start_module_compiler(module, scanner);
        }

        let intermediate = self.pass1(vm, env.clone())?;
        self.pass2(intermediate)?;

        {
            let mut env_borrow = env.borrow_mut();
            let module = env_borrow.get_module_mut(&module_name).unwrap();
            self.finish_module_compiler(module)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
