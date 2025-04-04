use clap::Parser;
use clap_verbosity_flag::Verbosity;
use human_panic::setup_panic;
use std::path::PathBuf;

use tardi::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    if let Some(file) = args.file {
        tardi::run_file(&file, args.print_stack)?;
    } else {
        tardi::repl()?;
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
