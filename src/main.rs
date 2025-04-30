use clap::Parser;
use clap_verbosity_flag::Verbosity;
use directories::ProjectDirs;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use human_panic::setup_panic;
use std::fs::read_to_string;
use std::path::PathBuf;
use tardi::config::Config;

use tardi::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .parse_env("TARDI_LOG")
        .filter_level(args.verbose.log_level_filter())
        .init();

    // TODO: some way to dump out the default config and print the path
    // TODO: can I use the Cli struct as a provider?
    // TODO: different qualifier and organization
    let project_dirs = ProjectDirs::from("", "", "Tardi");
    // Config file is from CLI args or the standard platform configuration
    let config_file = args
        .config
        .or_else(|| project_dirs.map(|pd| pd.config_dir().join("tardi.toml").to_owned()));
    let mut figment = Figment::from(Serialized::defaults(Config::default()));
    if let Some(config_file) = config_file {
        log::info!("config location: {}", config_file.display());
        figment = figment.merge(Toml::file(config_file));
    } else {
        log::warn!("no config file specified");
    }
    figment = figment.merge(Env::prefixed("TARDI_"));
    let config: Config = figment.extract()?;
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
