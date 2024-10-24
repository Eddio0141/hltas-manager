use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use sha2::Digest;

use crate::cfg;

pub fn try_root_dir() -> Result<PathBuf> {
    let working_dir = std::env::current_dir().context("Failed to get current dir")?;

    let root_cfg_path = working_dir.join(cfg::cfg_file_name());

    if root_cfg_path.is_file() {
        Ok(working_dir)
    } else {
        let tas_dir = working_dir.parent().context("Failed to get projects dir")?;
        let root_dir = tas_dir.parent().context("Failed to get root dir")?;
        let root_cfg_path = root_dir.join(cfg::cfg_file_name());

        if root_cfg_path.is_file() {
            Ok(root_dir.to_path_buf())
        } else {
            bail!("Using the program from an unknown location");
        }
    }
}

pub fn exe_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().context("Failed to get current exe path")?;
    Ok(exe_path
        .parent()
        .context("Failed to get exe dir")?
        .to_path_buf())
}

pub fn cfg_dir() -> Result<PathBuf> {
    let root = exe_dir()?;
    let file_name = cfg::cfg_file_name();

    Ok(root.join(file_name))
}

pub fn sha_256_file<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let file = fs::read(&path)
        .with_context(|| format!("Failed to open file {}", path.as_ref().display()))?;
    let mut hasher = sha2::Sha256::new();

    hasher.update(&file);

    Ok(hasher.finalize().to_vec())
}

pub fn force_link<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    if link.as_ref().is_file() {
        fs::remove_file(&link)?;
    }
    fs::hard_link(original, link)
}
