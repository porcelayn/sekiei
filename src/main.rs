mod build;
mod file_ops;
mod listing;
mod markdown;
mod paths;
mod serve;
mod utils;

use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser)]
#[clap(name = "sekiei")]
#[clap(about = "A simple static site generator", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build,
    Serve,
    RebuildAndServe,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => build::build()?,
        Commands::Serve => serve::serve().await?,
        Commands::RebuildAndServe => {
            build::build()?;
            serve::serve().await?;
        }
    }

    Ok(())
}