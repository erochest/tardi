
use clap::Parser;
use clap_verbosity_flag::Verbosity;
use env_logger;
use human_panic::setup_panic;

use tardi::error::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let script_path = &args.script_file;
    let script_text = std::fs::read_to_string(script_path)?;

    if args.print_stack {
        todo!("print stack");
    }
    println!("{:?}", args);

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