use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cfg;

pub fn root_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().context("Failed to get current exe path")?;
    Ok(exe_path
        .parent()
        .context("Failed to get exe dir")?
        .to_path_buf())
}

pub fn half_life_dir() -> Result<PathBuf> {
    let root = root_dir()?;
    let half_life_dir = "Half-Life";

    Ok(root.join(half_life_dir))
}

pub fn cfg_dir() -> Result<PathBuf> {
    let root = root_dir()?;
    let file_name = cfg::cfg_file_name();

    Ok(root.join(file_name))
}
