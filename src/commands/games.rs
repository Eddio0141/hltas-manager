use std::{fs, path::Path};

use anyhow::{Context, Result};

pub fn games<P>(half_life_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let games = games_in_dir(half_life_dir)?;

    for game in games {
        println!("{}", game);
    }

    Ok(())
}

pub fn games_in_dir<P>(dir: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    // TODO return references??
    let dir = dir.as_ref();

    let mut games = Vec::new();

    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory")?;
        let path = entry.path();

        if path.is_dir() && is_dir_game(&path) {
            let name = path.file_name().unwrap();
            games.push(name.to_string_lossy().to_string());
        }
    }

    Ok(games)
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
