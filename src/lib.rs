//! Tardi programming language implementation

/// Re-export error types
pub mod error;

/// Virtual Machine implementation
pub mod vm;

/// Scanner implementation
pub mod scanner;

/// Compiler implementation
pub mod compiler;

// Re-exports
pub use compiler::Compiler;
pub use error::Result;
pub use scanner::Scanner;
pub use vm::value::Value;
pub use vm::VM;

/// Run a Tardi source file
pub fn run_file(path: &PathBuf, print_stack: bool) -> Result<()> {
    let source = std::fs::read_to_string(path)?;
    let scanner = Scanner::new(&source);
    let mut compiler = Compiler::new();
    let program = compiler.compile(scanner)?;

    let mut vm = VM::new();
    vm.load_program(Box::new(program));
    vm.run()?;

    if print_stack {
        // Print stack contents from top to bottom
        for value in vm.stack_iter() {
            eprintln!("{}", value);
        }
    }

    Ok(())
}
