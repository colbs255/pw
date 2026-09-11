use crate::clipboard::copy as copy_to_clipboard;
use crate::password::generate_password;
use crate::paths::{identity_path, store_dir};
use crate::store::{
    build_entry_path, copy_entry, find_entries, get_entry, list_entries, move_entry, put_entry,
    remove_entry,
};
use anyhow::{Result, bail};
use std::io::{self, Write};
use std::path::Path;

const DEFAULT_GENERATED_LENGTH: usize = 25;

pub(crate) fn put(name: &str, force: bool) -> Result<()> {
    let store_dir = store_dir()?;
    let identity_path = identity_path()?;
    let path = build_entry_path(&store_dir, name)?;
    if !confirm_overwrite(&path, name, force)? {
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

    put_entry(&store_dir, &identity_path, name, &password)?;
    println!("saved {name}");
    Ok(())
}

pub(crate) fn generate(
    name: &str,
    length: Option<usize>,
    no_symbols: bool,
    force: bool,
) -> Result<()> {
    let length = length.unwrap_or(DEFAULT_GENERATED_LENGTH);
    if length == 0 {
        bail!("length must be greater than 0");
    }

    let store_dir = store_dir()?;
    let identity_path = identity_path()?;
    let path = build_entry_path(&store_dir, name)?;
    if !confirm_overwrite(&path, name, force)? {
        println!("aborted");
        return Ok(());
    }

    let password = generate_password(length, no_symbols);
    put_entry(&store_dir, &identity_path, name, &password)?;
    println!("saved {name}");
    println!("{password}");
    Ok(())
}

pub(crate) fn list() -> Result<()> {
    for name in list_entries(&store_dir()?)? {
        println!("{name}");
    }
    Ok(())
}

pub(crate) fn find(pattern: &str) -> Result<()> {
    for name in find_entries(&store_dir()?, pattern)? {
        println!("{name}");
    }
    Ok(())
}

pub(crate) fn get(name: &str, clipboard: bool) -> Result<()> {
    let password = get_entry(&store_dir()?, &identity_path()?, name)?;
    if clipboard {
        copy_to_clipboard(&password)?;
    } else {
        println!("{password}");
    }
    Ok(())
}

pub(crate) fn copy(old_name: &str, new_name: &str, force: bool) -> Result<()> {
    let store_dir = store_dir()?;
    let old_path = build_entry_path(&store_dir, old_name)?;
    if !old_path.exists() {
        bail!("no entry named {old_name}");
    }
    let new_path = build_entry_path(&store_dir, new_name)?;
    if !confirm_overwrite(&new_path, new_name, force)? {
        println!("aborted");
        return Ok(());
    }

    copy_entry(&store_dir, old_name, new_name)?;
    println!("copied {old_name} to {new_name}");
    Ok(())
}

pub(crate) fn mv(old_name: &str, new_name: &str, force: bool) -> Result<()> {
    let store_dir = store_dir()?;
    let old_path = build_entry_path(&store_dir, old_name)?;
    if !old_path.exists() {
        bail!("no entry named {old_name}");
    }
    let new_path = build_entry_path(&store_dir, new_name)?;
    if !confirm_overwrite(&new_path, new_name, force)? {
        println!("aborted");
        return Ok(());
    }

    move_entry(&store_dir, old_name, new_name)?;
    println!("moved {old_name} to {new_name}");
    Ok(())
}

pub(crate) fn remove(name: &str) -> Result<()> {
    let store_dir = store_dir()?;
    let path = build_entry_path(&store_dir, name)?;
    if !path.exists() {
        bail!("no entry named {name}");
    }
    if !confirm(&format!("remove {name}?"))? {
        println!("aborted");
        return Ok(());
    }
    remove_entry(&store_dir, name)?;
    println!("removed {name}");
    Ok(())
}

/// Returns whether it's OK to proceed writing to `path`: true if it doesn't
/// exist yet, `force` is set, or the user confirms the overwrite prompt.
fn confirm_overwrite(path: &Path, name: &str, force: bool) -> Result<bool> {
    if path.exists() && !force {
        return confirm(&format!("{name} already exists. Overwrite?"));
    }
    Ok(true)
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}
