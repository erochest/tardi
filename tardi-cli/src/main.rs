use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use human_panic::setup_panic;

use tardi_core::error::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    if let Some(command) = args.command {
        match command {
            Command::Scan { input_file } => {
                log::info!("scanning {:?}", input_file);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Scan a file and print the tokens it contains.
    Scan {
        /// The input file to scan.
        input_file: PathBuf,
    },
}
