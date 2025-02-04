use clap::Parser;
use clap_verbosity_flag::Verbosity;
use env_logger;
use human_panic::setup_panic;
use std::convert::TryFrom;

use tardi::run_file;
use tardi::error::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    if let Some(script_file) = args.script_file {
        run_file(&script_file, args.print_stack)?;
    } else {
        run_repl(args.print_stack)?;
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    /// The Tardi script file to execute. If not provided, runs the REPL.
    script_file: Option<std::path::PathBuf>,

    /// Print the stack after execution 
    #[arg(long)]
    print_stack: bool,
}
