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
    /// Insert a new password entry
    Insert { name: String },
    /// Print a password entry
    Get { name: String },
    /// Remove a password entry
    Remove { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Insert { name } => commands::insert(&name),
        Command::Get { name } => commands::get(&name),
        Command::Remove { name } => commands::remove(&name),
    }
}
