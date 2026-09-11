use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

pub(crate) fn store_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.data_dir().join("store"))
}

pub(crate) fn identity_path() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().join("key.txt"))
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "pw").context("could not determine home directory")
}
