use crate::paths::{identity_path, store_dir};
use crate::store::{build_entry_path, get_entry, insert_entry, list_entries, remove_entry};
use anyhow::{Result, bail};
use std::io::{self, Write};

pub(crate) fn insert(name: &str) -> Result<()> {
    let store_dir = store_dir()?;
    let identity_path = identity_path()?;
    let path = build_entry_path(&store_dir, name)?;
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

    insert_entry(&store_dir, &identity_path, name, &password)?;
    println!("saved {name}");
    Ok(())
}

pub(crate) fn list() -> Result<()> {
    for name in list_entries(&store_dir()?)? {
        println!("{name}");
    }
    Ok(())
}

pub(crate) fn get(name: &str) -> Result<()> {
    let password = get_entry(&store_dir()?, &identity_path()?, name)?;
    println!("{password}");
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

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}
