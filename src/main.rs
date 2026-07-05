use clap::Parser;
use clap_verbosity_flag::Verbosity;
use directories::ProjectDirs;
use figment::Figment;
use figment::providers::Serialized;
use human_panic::setup_panic;
use std::fs;
use std::path::{Path, PathBuf};
use tardi::config::Config;

use tardi::error::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .parse_env("TARDI_LOG")
        .filter_level(args.verbose.log_level_filter())
        .init();

    // TODO: some way to edit config from the command line
    // let config = read_config_sources(&args.config.as_deref())?;
    // if let Some(history_dir) = config.repl.history_file.as_ref().and_then(|p| p.parent()) {
    //     fs::create_dir_all(history_dir)?;
    // }

    // log::info!("config {:?}", config);

    match args.command {
        Some(Commands::Evaluate { script_files }) => {
            for file in script_files {
                // tardi::run_file(&file, &config, args.print_stack)?;
            }
            Ok(())
        }
        Some(Commands::Repl) => todo!(), // tardi::repl(&config),
        Some(Commands::ConfigInit) => {
            // let path = init_default_config()?;
            let path: PathBuf = todo!();
            println!("{}", path.display());
            Ok(())
        }
        None => {
            if let Some(file) = args.file {
                // tardi::run_file(&file, &config, args.print_stack)
            } else {
                // tardi::repl(&config)
            }
            Ok(())
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

fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "Tardi")
}

// TODO: can I use the Cli struct as a provider?
// TODO: different qualifier and organization
/// This reads the configuration from a file and runs it through a standard
/// set of configuration locations.
///
/// 1. Defaults;
/// 2. The TOML configuration file passed in, which defaults to a platform-
///    appropriate value, if that exists;
/// 3. Environment variables that begin with "TARDI_".
///
/// Arguably this should be handled by the command-line executable, not
/// by the library, but often the VM needs to handle this consistently.
/// Plus, putting it here is more testable.
pub fn read_config_sources(config_file: &Option<&Path>) -> Result<Config> {
    //     // Config file is from CLI args or the standard platform configuration
    //     let config_file = config_file
    //         .as_ref()
    //         .map(|path| path.to_path_buf())
    //         .or_else(default_config_file);

    // read from the sources
    let mut figment = Figment::from(Serialized::defaults(Config::default()));
    //     if let Some(config_file) = config_file {
    //         log::info!("config location: {}", config_file.display());
    //         figment = figment.admerge(Toml::file(config_file));
    //     } else {
    //         log::warn!("no config file specified");
    //     }
    //     figment = figment.admerge(Env::prefixed("TARDI_"));

    // extract the configuration
    let mut config: Config = figment.extract()?;
    log::debug!("configuration read: {:#?}", config);

    //     // patch the history_file
    //     let project_dirs = get_project_dirs();
    //     config.repl.history_file = config.repl.history_file.or_else(|| {
    //         project_dirs
    //             .as_ref()
    //             .map(|pd| pd.data_local_dir().join("repl-history.txt").to_path_buf())
    //     });

    Ok(config)
}

pub fn init_default_config() -> Result<PathBuf> {
    //     let config_file = default_config_file().ok_or(Error::MissingConfiguration)?;
    //
    //     if fs::exists(&config_file)? {
    //         log::warn!(
    //             "{} exists. not overwriting with default.",
    //             config_file.display()
    //         );
    //     } else {
    //         let contents = include_str!("./data/default_config.toml");
    //         fs::write(&config_file, contents)?;
    //     }
    //
    //     Ok(config_file)
    Ok("./tardi.toml".into())
}

fn default_config_file() -> Option<PathBuf> {
    //     let project_dirs = get_project_dirs();
    //     project_dirs
    //         .as_ref()
    //         .map(|pd| pd.config_dir().join("tardi.toml").to_owned())
    None
}

#[cfg(test)]
mod tests {
    use std::{env, str::FromStr};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    #[allow(clippy::result_large_err)]
    fn test_read_config_sources_reads_default_config_file_from_project_dirs() {
        let project = get_project_dirs().unwrap();
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                project.config_dir().join("Tardi"),
                r#"
            module_path = ["does/not/exist"]

            [repl]
            edit_mode = "Vi"
            history_file = "also/does/not/exist"
        "#,
            )
            .unwrap();

            let config = read_config_sources(&None);
            assert!(config.is_ok());
            let config = config.unwrap();
            assert!(
                config
                    .module_path
                    .contains(&project.data_dir().join("does/not/exist"))
            );

            Ok(())
        });
    }

    #[test]
    fn test_read_config_sources_reads_config_file_parameter() {
        let path = Path::new("tests/fixtures/tardi.toml");

        let result = read_config_sources(&Some(path));
        assert!(result.is_ok(), "error: {:?}", result);
        let config = result.unwrap();
    }

    // TODO: default config file passed in (default_config_file)
    // TODO: no config file (Config::default())
    // TODO: env variables
    // TODO: repl history file

    #[test]
    fn test_config_module_paths_accumulate() {
        let pwd = env::current_dir().unwrap();
        let path = Path::new("tests/fixtures/tardi.toml");

        let result = read_config_sources(&Some(path));
        assert!(result.is_ok(), "error: {:?}", result);
        let config = result.unwrap();

        assert!(config.module_path.contains(&pwd));
        assert!(
            config
                .module_path
                .contains(&PathBuf::from_str("tests/modules").unwrap())
        );
    }
}
