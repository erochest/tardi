use clap::Parser;
use clap_verbosity_flag::Verbosity;
use env_logger;
use human_panic::setup_panic;
use std::convert::TryFrom;

use tardi::compiler::compile;
use tardi::error::{Error, Result};
use tardi::parser::parse;
use tardi::vm::VM;


fn run_file(file_path: &std::path::Path, print_stack: bool) -> Result<()> {
    let script_text = std::fs::read_to_string(file_path)?;

    let tokens = parse(&script_text).into_iter().collect::<Result<Vec<_>>>()?;
    let chunk = compile(tokens);
    let mut vm = VM::new();
    vm.execute(chunk)?;

    if print_stack {
        vm.print_stack();
    }

    Ok(())
}

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    run_file(&args.script_file, args.print_stack)?;

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    /// The Tardi script file to execute
    script_file: std::path::PathBuf,

    /// Print the stack after execution 
    #[arg(long)]
    print_stack: bool,
}
