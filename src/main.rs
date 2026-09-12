mod commands;
mod identity;
mod password;
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
    Put {
        name: String,
        /// Overwrite an existing entry without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Generate a random password and store it, overwriting any existing
    /// entry with the same name
    Generate {
        name: String,
        /// Length of the generated password
        length: Option<usize>,
        /// Exclude symbols, using only letters and digits
        #[arg(short = 'n', long)]
        no_symbols: bool,
        /// Overwrite an existing entry without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// List all password entries
    #[command(visible_alias = "ls")]
    List,
    /// List password entries whose name contains a pattern
    Find { pattern: String },
    /// Print a password entry
    Get { name: String },
    /// Edit a password entry in $EDITOR, creating it if it doesn't exist
    Edit { name: String },
    /// Copy a password entry to a new name, keeping the original
    #[command(visible_alias = "cp")]
    Copy {
        old_name: String,
        new_name: String,
        /// Overwrite an existing entry without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Rename a password entry
    #[command(visible_alias = "rename")]
    Mv {
        old_name: String,
        new_name: String,
        /// Overwrite an existing entry without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Remove a password entry, or a group of entries with --recursive
    #[command(visible_alias = "rm")]
    Remove {
        name: String,
        /// Remove a group of entries and everything under it
        #[arg(short, long)]
        recursive: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Put { name, force } => commands::put(&name, force),
        Command::Generate {
            name,
            length,
            no_symbols,
            force,
        } => commands::generate(&name, length, no_symbols, force),
        Command::List => commands::list(),
        Command::Find { pattern } => commands::find(&pattern),
        Command::Get { name } => commands::get(&name),
        Command::Edit { name } => commands::edit(&name),
        Command::Copy {
            old_name,
            new_name,
            force,
        } => commands::copy(&old_name, &new_name, force),
        Command::Mv {
            old_name,
            new_name,
            force,
        } => commands::mv(&old_name, &new_name, force),
        Command::Remove { name, recursive } => commands::remove(&name, recursive),
    }
}
