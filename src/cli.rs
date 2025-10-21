use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Server {
        #[arg(index = 1)]
        ip: String,
    },
    Client {
        #[arg(long, short, default_value = "unknown")]
        user: String,
        #[arg(index = 1)]
        connection: String,
    },
}
