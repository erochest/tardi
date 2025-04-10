use crate::compiler::Compiler;
use crate::env::Environment;
use crate::error::Result;
use crate::scanner::{Scanner, Token, TokenType};
use crate::shared::{shared, Shared};
use crate::value::{Callable, Function, Value};
use crate::vm::{OpCode, OpFn, VM};

pub trait Scan {
    fn scan(&mut self, input: &str) -> Result<Vec<Result<Token>>>;
    fn set_source(&mut self, input: &str);
    fn scan_token(&mut self) -> Option<Result<Token>>;
    fn scan_tokens_until(&mut self, token_type: TokenType) -> Result<Vec<Result<Token>>>;
    fn read_string_until(&mut self, delimiter: &str) -> Result<String>;
}

pub trait Compile {
    fn compile<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        input: &str,
    ) -> Result<()>;
    fn compile_lambda<E: Execute>(
        &mut self,
        executor: &mut E,
        env: Shared<Environment>,
        scanner: &mut Scanner,
        input: &str,
    ) -> Result<()>;
}

pub trait Execute {
    fn run(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        scanner: &mut Scanner,
    ) -> Result<()>;
    fn stack(&self) -> Vec<Value>;
    fn execute_macro(
        &mut self,
        env: Shared<Environment>,
        compiler: &mut Compiler,
        scanner: &mut Scanner,
        trigger: &TokenType,
        function: &Function,
        tokens: &[Value],
    ) -> Result<Vec<Value>>;
}

// TODO: make the VM the orchestrator and get rid of this?
pub struct Tardi {
    pub input: Option<String>,
    pub environment: Shared<Environment>,
    pub scanner: Scanner,
    pub compiler: Compiler,
    pub executor: VM,
}

impl Tardi {
    pub fn new(
        environment: Environment,
        scanner: Scanner,
        compiler: Compiler,
        executor: VM,
    ) -> Self {
        Tardi {
            input: None,
            environment: shared(environment),
            scanner,
            compiler,
            executor,
        }
    }

    pub fn reset(&mut self) {
        self.input = None;
    }

    pub fn scan_str(&mut self, input: &str) -> Result<Vec<Result<Token>>> {
        log::debug!("input : {:?}", input);
        let input = input.to_string();
        self.input = Some(input);
        Scan::scan(&mut self.scanner, self.input.as_ref().unwrap())
    }

    pub fn compile(&mut self, input: &str) -> Result<Shared<Environment>> {
        self.inject_macro_readers()?;
        self.compiler.compile(
            &mut self.executor,
            self.environment.clone(),
            &mut self.scanner,
            input,
        )?;
        Ok(self.environment.clone())
    }

    pub fn execute(&mut self) -> Result<()> {
        log::debug!("environment:\n{:?}", self.environment.borrow());
        self.executor.run(
            self.environment.clone(),
            &mut self.compiler,
            &mut self.scanner,
        )
    }

    pub fn execute_str(&mut self, input: &str) -> Result<()> {
        self.reset();
        self.compile(input)?;
        self.execute()
    }

    pub fn stack(&self) -> Vec<Value> {
        self.executor.stack()
    }

    fn inject_macro_readers(&mut self) -> Result<()> {
        let env = self.environment.borrow_mut();
        let op_map = env.get_op_map();

        // if !op_map.contains_key("scan-token") {
        //     let callable = Callable::Fn(todo!("scan-token implementation"));
        //     env.add_to_op_table(shared(callable));
        // }

        Ok(())
    }

    pub(crate) fn execute_ip(&mut self, ip: usize) -> Result<()> {
        let bookmark = self.executor.ip;
        self.executor.ip = ip;
        self.execute()?;
        self.executor.ip = bookmark;
        Ok(())
    }
}

impl Default for Tardi {
    fn default() -> Tardi {
        let environment = Environment::with_builtins();
        let scanner = Scanner::default();
        let compiler = Compiler::default();
        let executor = VM::new();
        Tardi::new(environment, scanner, compiler, executor)
    }
}

// Create the default operation table
pub fn create_op_table() -> Vec<Shared<Callable>> {
    let size = OpCode::StringConcat as usize + 1;
    let mut op_table = Vec::with_capacity(size);

    // Set up the operation table
    push_op(&mut op_table, lit);
    push_op(&mut op_table, dup);
    push_op(&mut op_table, swap);
    push_op(&mut op_table, rot);
    push_op(&mut op_table, drop_op);
    push_op(&mut op_table, stack_size);
    push_op(&mut op_table, add);
    push_op(&mut op_table, subtract);
    push_op(&mut op_table, multiply);
    push_op(&mut op_table, divide);
    push_op(&mut op_table, equal);
    push_op(&mut op_table, less);
    push_op(&mut op_table, greater);
    push_op(&mut op_table, not);
    push_op(&mut op_table, to_r);
    push_op(&mut op_table, r_from);
    push_op(&mut op_table, r_fetch);
    push_op(&mut op_table, create_list);
    push_op(&mut op_table, append);
    push_op(&mut op_table, prepend);
    push_op(&mut op_table, concat);
    push_op(&mut op_table, split_head);
    push_op(&mut op_table, create_string);
    push_op(&mut op_table, to_string);
    push_op(&mut op_table, utf8_to_string);
    push_op(&mut op_table, string_concat);
    push_op(&mut op_table, call);
    push_op(&mut op_table, call_stack);
    push_op(&mut op_table, return_op);
    push_op(&mut op_table, jump);
    push_op(&mut op_table, jump_stack);
    push_op(&mut op_table, function);

    op_table
}

fn push_op(op_table: &mut Vec<Shared<Callable>>, op: OpFn) {
    op_table.push(shared(Callable::BuiltIn(op)));
}

// Helper function to add an operation to the table and map
// Will be used when we implement function support
// fn add_word(op_table: &mut Vec<OpFn>, op_map: &mut HashMap<String, usize>, op: OpFn, name: &str) {
//     let index = op_table.len();
//     op_table.push(op);
//     op_map.insert(name.to_string(), index);
// }

// Define the operations
pub fn lit(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.lit()
}

pub fn dup(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.dup()
}

pub fn swap(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.swap()
}

pub fn rot(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.rot()
}

pub fn drop_op(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.drop_op()
}

pub fn stack_size(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.stack_size_op()
}

pub fn add(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.add()
}

pub fn subtract(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.subtract()
}

pub fn multiply(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.multiply()
}

pub fn divide(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.divide()
}

pub fn to_r(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.to_r()
}

pub fn r_from(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.r_from()
}

pub fn r_fetch(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.r_fetch()
}

pub fn not(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.not()
}

pub fn equal(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.equal()
}

pub fn less(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.less()
}

pub fn greater(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.greater()
}

// List operations
pub fn create_list(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.create_list()
}

pub fn append(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.append()
}

pub fn prepend(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.prepend()
}

pub fn concat(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.concat()
}

pub fn split_head(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.split_head()
}

// String operations
pub fn create_string(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.create_string()
}

pub fn to_string(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.to_string()
}

pub fn utf8_to_string(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.utf8_to_string()
}

pub fn string_concat(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.string_concat()
}

// Function operations
pub fn call(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.call()
}

pub fn call_stack(vm: &mut VM, compiler: &mut Compiler, scanner: &mut Scanner) -> Result<()> {
    vm.call_stack(compiler, scanner)
}

pub fn return_op(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.return_op()
}

pub fn jump(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.jump()
}

pub fn jump_stack(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.jump_stack()
}

pub fn function(vm: &mut VM, _compiler: &mut Compiler, _scanner: &mut Scanner) -> Result<()> {
    vm.function()
}
