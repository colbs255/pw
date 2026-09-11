use age::secrecy::ExposeSecret;
use age::x25519::Identity;
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
        Command::Insert { name } => insert(&name),
        Command::Get { name } => get(&name),
        Command::Remove { name } => remove(&name),
    }
}

fn insert(name: &str) -> Result<()> {
    let path = entry_path(name)?;
    if path.exists() && !confirm(&format!("{name} already exists. Overwrite?"))? {
        println!("aborted");
        return Ok(());
    }

    let password = {
        let password = rpassword::prompt_password("Password: ")?;
        let confirmed = rpassword::prompt_password("Confirm: ")?;
        if password != confirmed {
            bail!("passwords did not match");
        }
        password
    };

    let identity = load_or_create_identity()?;
    let ciphertext =
        age::encrypt(&identity.to_public(), password.as_bytes()).context("encrypting entry")?;

    fs::create_dir_all(store_dir()?)?;
    fs::write(&path, ciphertext).with_context(|| format!("writing {}", path.display()))?;
    println!("saved {name}");
    Ok(())
}

fn get(name: &str) -> Result<()> {
    let path = entry_path(name)?;
    if !path.exists() {
        bail!("no entry named {name}");
    }

    let identity = load_identity()?;
    let ciphertext = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let plaintext = age::decrypt(&identity, &ciphertext).context("decrypting entry")?;
    let password = String::from_utf8(plaintext).context("decrypted entry was not valid UTF-8")?;
    println!("{password}");
    Ok(())
}

fn remove(name: &str) -> Result<()> {
    let path = entry_path(name)?;
    if !path.exists() {
        bail!("no entry named {name}");
    }
    if !confirm(&format!("remove {name}?"))? {
        println!("aborted");
        return Ok(());
    }
    fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    println!("removed {name}");
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

/// Loads the local age identity (private key), generating one on first use.
fn load_or_create_identity() -> Result<Identity> {
    let path = identity_path()?;

    if path.exists() {
        return parse_identity_file(&path);
    }

    let identity = Identity::generate();
    let recipient = identity.to_public();
    let secret = identity.to_string();
    fs::create_dir_all(project_dirs()?.config_dir())?;
    fs::write(
        &path,
        format!(
            "# pw private key - keep this secret\n# public key: {recipient}\n{}\n",
            secret.expose_secret()
        ),
    )
    .with_context(|| format!("writing {}", path.display()))?;
    restrict_permissions(&path)?;
    eprintln!("generated new key at {}", path.display());
    eprintln!("public key: {recipient}");
    Ok(identity)
}

/// Loads the existing local age identity, erroring if none has been created yet.
fn load_identity() -> Result<Identity> {
    let path = identity_path()?;
    if !path.exists() {
        bail!(
            "no key file at {}; insert an entry first to generate one",
            path.display()
        );
    }
    parse_identity_file(&path)
}

/// Parses the age identity (private key) out of an existing key file.
fn parse_identity_file(path: &Path) -> Result<Identity> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let key_line = contents
        .lines()
        .find(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .context("key file has no key")?;
    Identity::from_str(key_line.trim())
        .map_err(|e| anyhow::anyhow!("parsing {}: {e}", path.display()))
}

fn entry_path(name: &str) -> Result<PathBuf> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        bail!("invalid entry name: {name:?}");
    }
    Ok(store_dir()?.join(format!("{name}.age")))
}

fn store_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.data_dir().join("store"))
}

fn identity_path() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().join("key.txt"))
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "pw").context("could not determine home directory")
}
