use clap::Parser;
use clap_verbosity_flag::Verbosity;
use human_panic::setup_panic;

use tardi_core::error::Result;

fn main() -> Result<()> {
    setup_panic!();
    let args = Cli::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    println!("{:?}", args);

    Ok(())
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,
}
