use crate::identity::{load_identity, load_or_create_identity};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn build_entry_path(store_dir: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty() {
        bail!("invalid entry name: {name:?}");
    }

    let mut path = store_dir.to_path_buf();
    for segment in name.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." || segment.contains('\\') {
            bail!("invalid entry name: {name:?}");
        }
        path.push(segment);
    }
    path.set_extension("age");
    Ok(path)
}

/// Encrypts `password` under the local identity (generating one on first use)
/// and writes it to `<store_dir>/<name>.age`, creating parent dirs as needed.
pub(crate) fn insert_entry(
    store_dir: &Path,
    identity_path: &Path,
    name: &str,
    password: &str,
) -> Result<()> {
    let path = build_entry_path(store_dir, name)?;
    let identity = load_or_create_identity(identity_path)?;
    let ciphertext =
        age::encrypt(&identity.to_public(), password.as_bytes()).context("encrypting entry")?;

    fs::create_dir_all(path.parent().expect("entry paths always have a parent"))?;
    fs::write(&path, ciphertext).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Reads and decrypts `<store_dir>/<name>.age` using the local identity.
pub(crate) fn get_entry(store_dir: &Path, identity_path: &Path, name: &str) -> Result<String> {
    let path = build_entry_path(store_dir, name)?;
    if !path.exists() {
        bail!("no entry named {name}");
    }

    let identity = load_identity(identity_path)?;
    let ciphertext = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let plaintext = age::decrypt(&identity, &ciphertext).context("decrypting entry")?;
    String::from_utf8(plaintext).context("decrypted entry was not valid UTF-8")
}

/// Deletes `<store_dir>/<name>.age` and prunes any now-empty ancestor directories.
pub(crate) fn remove_entry(store_dir: &Path, name: &str) -> Result<()> {
    let path = build_entry_path(store_dir, name)?;
    if !path.exists() {
        bail!("no entry named {name}");
    }
    fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    prune_empty_parents(&path, store_dir);
    Ok(())
}

/// Removes now-empty ancestor directories of a just-deleted entry, up to (not
/// including) the store root, so nested entries don't leave empty folders behind.
fn prune_empty_parents(path: &Path, store_dir: &Path) {
    let mut dir = path.parent();
    while let Some(d) = dir {
        if d == store_dir || fs::remove_dir(d).is_err() {
            break;
        }
        dir = d.parent();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_entry_path_flat_name() {
        let path = build_entry_path(Path::new("/store"), "github").unwrap();
        assert_eq!(path, Path::new("/store/github.age"));
    }

    #[test]
    fn build_entry_path_nested_name() {
        let path = build_entry_path(Path::new("/store"), "github/key1").unwrap();
        assert_eq!(path, Path::new("/store/github/key1.age"));
    }

    #[test]
    fn build_entry_path_rejects_invalid_names() {
        for name in [
            "",
            "/",
            "..",
            ".",
            "../evil",
            "a/../../evil",
            "a/./b",
            "a//b",
            "a/",
            "/a",
            "a\\b",
        ] {
            assert!(
                build_entry_path(Path::new("/store"), name).is_err(),
                "expected {name:?} to be rejected"
            );
        }
    }

    #[test]
    fn prune_empty_parents_removes_empty_ancestors_up_to_store_root() {
        let store = tempfile::tempdir().unwrap();
        let nested = store.path().join("github").join("key1.age");
        fs::create_dir_all(nested.parent().unwrap()).unwrap();
        fs::write(&nested, b"ciphertext").unwrap();
        fs::remove_file(&nested).unwrap();

        prune_empty_parents(&nested, store.path());

        assert!(!store.path().join("github").exists());
        assert!(store.path().exists());
    }

    #[test]
    fn prune_empty_parents_stops_at_non_empty_directory() {
        let store = tempfile::tempdir().unwrap();
        let dir = store.path().join("github");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("key1.age"), b"ciphertext").unwrap();
        fs::write(dir.join("key2.age"), b"ciphertext").unwrap();
        fs::remove_file(dir.join("key2.age")).unwrap();

        prune_empty_parents(&dir.join("key2.age"), store.path());

        assert!(dir.exists());
        assert!(dir.join("key1.age").exists());
    }

    #[test]
    fn insert_then_get_roundtrip() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        insert_entry(store.path(), &key, "github", "hunter2").unwrap();

        assert_eq!(get_entry(store.path(), &key, "github").unwrap(), "hunter2");
    }

    #[test]
    fn insert_nested_entries_are_independent() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        insert_entry(store.path(), &key, "github/key1", "p1").unwrap();
        insert_entry(store.path(), &key, "github/key2", "p2").unwrap();

        assert_eq!(get_entry(store.path(), &key, "github/key1").unwrap(), "p1");
        assert_eq!(get_entry(store.path(), &key, "github/key2").unwrap(), "p2");
    }

    #[test]
    fn get_missing_entry_errors() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        assert!(get_entry(store.path(), &key, "nope").is_err());
    }

    #[test]
    fn get_without_identity_errors() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");
        // simulate an entry present without ever having generated a key
        fs::create_dir_all(store.path()).unwrap();
        fs::write(store.path().join("github.age"), b"not real ciphertext").unwrap();

        let err = get_entry(store.path(), &key, "github").unwrap_err();
        assert!(err.to_string().contains("no key file"));
    }

    #[test]
    fn remove_deletes_entry_and_prunes_empty_dir() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");
        insert_entry(store.path(), &key, "github/key1", "p1").unwrap();

        remove_entry(store.path(), "github/key1").unwrap();

        assert!(get_entry(store.path(), &key, "github/key1").is_err());
        assert!(!store.path().join("github").exists());
    }

    #[test]
    fn remove_missing_entry_errors() {
        let store = tempfile::tempdir().unwrap();
        assert!(remove_entry(store.path(), "nope").is_err());
    }
}
