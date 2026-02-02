use clap::Parser;

mod cli;
mod commands;
mod config;
mod database;
mod error;
mod migrations;
mod schema;

use cli::{Cli, Commands};
use error::CliResult;

#[tokio::main]
async fn main() -> CliResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => commands::init::run(args).await,
        Commands::Generate(args) => commands::generate::run(args).await,
        Commands::Migrate(args) => commands::migrate::run(args).await,
        Commands::Status(args) => commands::status::run(args).await,
        Commands::Destroy(args) => commands::destroy::run(args).await,
        Commands::Schema(args) => commands::schema::run(args).await,
    }
}
