mod commands;
mod identity;
mod paths;
mod store;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pw", about = "Encrypted password storage")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Store a password entry, overwriting any existing one with the same name
    Put { name: String },
    /// List all password entries
    #[command(visible_alias = "ls")]
    List,
    /// Print a password entry
    Get { name: String },
    /// Remove a password entry
    #[command(visible_alias = "rm")]
    Remove { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Put { name } => commands::put(&name),
        Command::List => commands::list(),
        Command::Get { name } => commands::get(&name),
        Command::Remove { name } => commands::remove(&name),
    }
}
