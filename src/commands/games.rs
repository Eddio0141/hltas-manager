use std::{fs, path::Path};

use anyhow::{Context, Result};

pub fn games<P>(half_life_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let half_life_dir = half_life_dir.as_ref();

    for entry in fs::read_dir(half_life_dir).context("Failed to read half-life directory")? {
        let entry = entry.context("Failed to read half-life directory")?;
        let path = entry.path();

        if path.is_dir() && is_dir_game(&path) {
            let name = path.file_name().unwrap();
            println!("{}", name.to_string_lossy());
        }
    }

    Ok(())
}

const HD_GAME: &str = "_hd";
const ADDON_GAME: &str = "_addon";
const DLLS_DIR: &str = "dlls";
const CL_DLLS_DIR: &str = "cl_dlls";

pub fn is_dir_game<P>(path: P) -> bool
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if path.is_file() {
        return false;
    }

    match path.file_name() {
        Some(file_name) => {
            let file_name = file_name.to_string_lossy();

            !file_name.ends_with(HD_GAME)
                && !file_name.ends_with(ADDON_GAME)
                && (path.join(DLLS_DIR).is_dir() || path.join(CL_DLLS_DIR).is_dir())
        }
        None => false,
    }
}
