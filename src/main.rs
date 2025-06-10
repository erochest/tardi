use clap::Parser;
use clap_verbosity_flag::Verbosity;
use human_panic::setup_panic;
use std::fs;
use std::path::PathBuf;
use tardi::config::{init_default_config, read_config_sources};

use tardi::error::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .parse_env("TARDI_LOG")
        .filter_level(args.verbose.log_level_filter())
        .init();

    // TODO: some way to edit config from the command line
    let config = read_config_sources(&args.config.as_deref())?;
    if let Some(history_dir) = config.repl.history_file.as_ref().and_then(|p| p.parent()) {
        fs::create_dir_all(history_dir)?;
    }

    log::info!("config {:?}", config);

    match args.command {
        Some(Commands::Evaluate { script_files }) => {
            for file in script_files {
                tardi::run_file(&file, &config, args.print_stack)?;
            }
            Ok(())
        }
        Some(Commands::Repl) => tardi::repl(&config),
        Some(Commands::ConfigInit) => {
            let path = init_default_config()?;
            println!("{}", path.display());
            Ok(())
        }
        None => {
            if let Some(file) = args.file {
                tardi::run_file(&file, &config, args.print_stack)
            } else {
                tardi::repl(&config)
            }
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    /// Print the contents of the stack when the program exits
    #[arg(long)]
    print_stack: bool,

    /// The location of the configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// The Tardi source file to execute
    file: Option<PathBuf>,
}

#[derive(Debug, Parser)]
enum Commands {
    /// Execute one or more files as scripts.
    Evaluate {
        /// The files to interpret and run.
        script_files: Vec<PathBuf>,
    },

    /// Run a REPL to execute Tardi interactively.
    Repl,

    /// Initialize configuration by outputting a default configuration
    /// and printing where it was output.
    ConfigInit,
}
