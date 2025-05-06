use clap::Parser;
use clap_verbosity_flag::Verbosity;
use human_panic::setup_panic;
use std::fs;
use std::path::PathBuf;
use tardi::config::read_config_sources;

use tardi::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .parse_env("TARDI_LOG")
        .filter_level(args.verbose.log_level_filter())
        .init();

    // TODO: some way to dump out the default config and print the path
    // TODO: some way to edit config from the command line
    let config = read_config_sources(&args.config.as_deref())?;
    if let Some(history_dir) = config.repl.history_file.as_ref().and_then(|p| p.parent()) {
        fs::create_dir_all(history_dir)?;
    }

    log::info!("config {:?}", config);

    if let Some(file) = args.file {
        tardi::run_file(&file, config, args.print_stack)?;
    } else {
        tardi::repl(config)?;
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

    /// The location of the configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,
}
