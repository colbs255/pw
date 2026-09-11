use age::secrecy::ExposeSecret;
use age::x25519::Identity;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Loads the local age identity (private key), generating one on first use.
pub(crate) fn load_or_create_identity(path: &Path) -> Result<Identity> {
    if path.exists() {
        return parse_identity_file(path);
    }

    let identity = Identity::generate();
    let recipient = identity.to_public();
    let secret = identity.to_string();
    fs::create_dir_all(path.parent().expect("identity path always has a parent"))?;
    fs::write(
        path,
        format!(
            "# pw private key - keep this secret\n# public key: {recipient}\n{}\n",
            secret.expose_secret()
        ),
    )
    .with_context(|| format!("writing {}", path.display()))?;
    restrict_permissions(path)?;
    eprintln!("generated new key at {}", path.display());
    eprintln!("public key: {recipient}");
    Ok(identity)
}

/// Loads the existing local age identity, erroring if none has been created yet.
pub(crate) fn load_identity(path: &Path) -> Result<Identity> {
    if !path.exists() {
        bail!(
            "no key file at {}; insert an entry first to generate one",
            path.display()
        );
    }
    parse_identity_file(path)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let identity = Identity::generate();
        let ciphertext = age::encrypt(&identity.to_public(), b"hunter2").unwrap();
        let plaintext = age::decrypt(&identity, &ciphertext).unwrap();
        assert_eq!(plaintext, b"hunter2");
    }

    #[test]
    fn decrypt_fails_with_wrong_identity() {
        let identity = Identity::generate();
        let other = Identity::generate();
        let ciphertext = age::encrypt(&identity.to_public(), b"hunter2").unwrap();
        assert!(age::decrypt(&other, &ciphertext).is_err());
    }

    #[test]
    fn load_or_create_generates_and_reuses_identity() {
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("key.txt");

        load_or_create_identity(&key).unwrap();
        let first = fs::read_to_string(&key).unwrap();
        load_or_create_identity(&key).unwrap();
        let second = fs::read_to_string(&key).unwrap();

        assert_eq!(first, second);
    }

    #[test]
    #[cfg(unix)]
    fn load_or_create_restricts_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("key.txt");

        load_or_create_identity(&key).unwrap();

        let mode = fs::metadata(&key).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn load_identity_errors_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("key.txt");
        assert!(load_identity(&key).is_err());
    }
}
