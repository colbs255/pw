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
    /// Remove a password entry
    Remove { name: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Insert { name } => {
            println!("insert: {name} (not yet implemented)");
        }
        Command::Remove { name } => {
            println!("remove: {name} (not yet implemented)");
        }
    }
}
