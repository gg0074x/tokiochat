use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::client::client;
use crate::server::server;

mod cli;
mod client;
mod server;
mod structs;
mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Server { ip } => server(ip).await?,
        Commands::Client { connection, user } => client(connection, user).await?,
    }
    Ok(())
}
