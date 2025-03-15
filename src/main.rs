
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use env_logger;
use human_panic::setup_panic;
use std::path::PathBuf;

use tardi::{Result, Scanner, Compiler, VM};

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    if let Some(file) = args.file {
        run_file(&file, args.print_stack)?;
    }

    Ok(())
}

// TODO: move this to `lib.rs`
fn run_file(path: &PathBuf, print_stack: bool) -> Result<()> {
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

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    /// The Tardi source file to execute
    file: Option<PathBuf>,

    /// Print the contents of the stack when the program exits
    #[arg(long)]
    print_stack: bool,
}
