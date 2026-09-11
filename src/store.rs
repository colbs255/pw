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
pub(crate) fn put_entry(
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

/// Lists all entry names under the store directory, sorted alphabetically.
pub(crate) fn list_entries(store_dir: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    if store_dir.exists() {
        collect_entries(store_dir, store_dir, &mut names)?;
    }
    names.sort();
    Ok(names)
}

/// Lists entry names under the store directory whose name contains `pattern`
/// (case-insensitive), sorted alphabetically.
pub(crate) fn find_entries(store_dir: &Path, pattern: &str) -> Result<Vec<String>> {
    let pattern = pattern.to_lowercase();
    Ok(list_entries(store_dir)?
        .into_iter()
        .filter(|name| name.to_lowercase().contains(&pattern))
        .collect())
}

fn collect_entries(store_dir: &Path, dir: &Path, names: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            collect_entries(store_dir, &path, names)?;
        } else if path.extension() == Some(std::ffi::OsStr::new("age")) {
            let relative = path
                .strip_prefix(store_dir)
                .expect("entry is under store_dir")
                .with_extension("");
            let name = relative
                .iter()
                .map(|c| c.to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            names.push(name);
        }
    }
    Ok(())
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

/// Renames `<store_dir>/<old_name>.age` to `<store_dir>/<new_name>.age`,
/// creating parent dirs for the new name and pruning any now-empty ancestor
/// directories of the old one.
pub(crate) fn move_entry(store_dir: &Path, old_name: &str, new_name: &str) -> Result<()> {
    let old_path = build_entry_path(store_dir, old_name)?;
    if !old_path.exists() {
        bail!("no entry named {old_name}");
    }
    let new_path = build_entry_path(store_dir, new_name)?;

    fs::create_dir_all(new_path.parent().expect("entry paths always have a parent"))?;
    fs::rename(&old_path, &new_path)
        .with_context(|| format!("renaming {} to {}", old_path.display(), new_path.display()))?;
    prune_empty_parents(&old_path, store_dir);
    Ok(())
}

/// Copies `<store_dir>/<old_name>.age` to `<store_dir>/<new_name>.age`,
/// creating parent dirs for the new name, leaving the old entry in place.
pub(crate) fn copy_entry(store_dir: &Path, old_name: &str, new_name: &str) -> Result<()> {
    let old_path = build_entry_path(store_dir, old_name)?;
    if !old_path.exists() {
        bail!("no entry named {old_name}");
    }
    let new_path = build_entry_path(store_dir, new_name)?;

    fs::create_dir_all(new_path.parent().expect("entry paths always have a parent"))?;
    fs::copy(&old_path, &new_path)
        .with_context(|| format!("copying {} to {}", old_path.display(), new_path.display()))?;
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
    fn list_entries_empty_when_store_dir_missing() {
        let store = tempfile::tempdir().unwrap();
        let missing = store.path().join("store");

        assert_eq!(list_entries(&missing).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn list_entries_returns_sorted_flat_and_nested_names() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        put_entry(store.path(), &key, "zebra", "p1").unwrap();
        put_entry(store.path(), &key, "github/key1", "p2").unwrap();
        put_entry(store.path(), &key, "github/key2", "p3").unwrap();
        put_entry(store.path(), &key, "apple", "p4").unwrap();

        assert_eq!(
            list_entries(store.path()).unwrap(),
            vec!["apple", "github/key1", "github/key2", "zebra"]
        );
    }

    #[test]
    fn find_entries_matches_substring_case_insensitively() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        put_entry(store.path(), &key, "zebra", "p1").unwrap();
        put_entry(store.path(), &key, "github/key1", "p2").unwrap();
        put_entry(store.path(), &key, "github/key2", "p3").unwrap();
        put_entry(store.path(), &key, "apple", "p4").unwrap();

        assert_eq!(
            find_entries(store.path(), "GitHub").unwrap(),
            vec!["github/key1", "github/key2"]
        );
    }

    #[test]
    fn find_entries_empty_when_no_match() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        put_entry(store.path(), &key, "apple", "p1").unwrap();

        assert_eq!(
            find_entries(store.path(), "nope").unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn put_then_get_roundtrip() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        put_entry(store.path(), &key, "github", "hunter2").unwrap();

        assert!(store.path().join("github.age").exists());
        assert_eq!(get_entry(store.path(), &key, "github").unwrap(), "hunter2");
    }

    #[test]
    fn put_nested_entries_are_independent() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");

        put_entry(store.path(), &key, "github/key1", "p1").unwrap();
        put_entry(store.path(), &key, "github/key2", "p2").unwrap();

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
        put_entry(store.path(), &key, "github/key1", "p1").unwrap();
        assert!(store.path().join("github/key1.age").exists());

        remove_entry(store.path(), "github/key1").unwrap();

        assert!(!store.path().join("github/key1.age").exists());
        assert!(!store.path().join("github").exists());
        assert!(get_entry(store.path(), &key, "github/key1").is_err());
    }

    #[test]
    fn remove_missing_entry_errors() {
        let store = tempfile::tempdir().unwrap();
        assert!(remove_entry(store.path(), "nope").is_err());
    }

    #[test]
    fn move_entry_renames_and_preserves_content() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");
        put_entry(store.path(), &key, "old", "hunter2").unwrap();

        move_entry(store.path(), "old", "new").unwrap();

        assert!(!store.path().join("old.age").exists());
        assert_eq!(get_entry(store.path(), &key, "new").unwrap(), "hunter2");
    }

    #[test]
    fn move_entry_creates_new_parent_dirs() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");
        put_entry(store.path(), &key, "flat", "p1").unwrap();

        move_entry(store.path(), "flat", "github/nested").unwrap();

        assert_eq!(
            get_entry(store.path(), &key, "github/nested").unwrap(),
            "p1"
        );
    }

    #[test]
    fn move_entry_prunes_empty_old_parent_dir() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");
        put_entry(store.path(), &key, "github/key1", "p1").unwrap();

        move_entry(store.path(), "github/key1", "flat").unwrap();

        assert!(!store.path().join("github").exists());
        assert_eq!(get_entry(store.path(), &key, "flat").unwrap(), "p1");
    }

    #[test]
    fn move_entry_missing_source_errors() {
        let store = tempfile::tempdir().unwrap();
        assert!(move_entry(store.path(), "nope", "new").is_err());
    }

    #[test]
    fn copy_entry_duplicates_and_preserves_original() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");
        put_entry(store.path(), &key, "old", "hunter2").unwrap();

        copy_entry(store.path(), "old", "new").unwrap();

        assert_eq!(get_entry(store.path(), &key, "old").unwrap(), "hunter2");
        assert_eq!(get_entry(store.path(), &key, "new").unwrap(), "hunter2");
    }

    #[test]
    fn copy_entry_creates_new_parent_dirs() {
        let store = tempfile::tempdir().unwrap();
        let key = store.path().join("key.txt");
        put_entry(store.path(), &key, "flat", "p1").unwrap();

        copy_entry(store.path(), "flat", "github/nested").unwrap();

        assert_eq!(
            get_entry(store.path(), &key, "github/nested").unwrap(),
            "p1"
        );
    }

    #[test]
    fn copy_entry_missing_source_errors() {
        let store = tempfile::tempdir().unwrap();
        assert!(copy_entry(store.path(), "nope", "new").is_err());
    }
}
