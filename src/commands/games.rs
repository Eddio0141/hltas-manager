use std::{fs, path::Path};

use anyhow::{Context, Result};
use log::info;

pub fn games<P>(half_life_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let games = game_dir_types(half_life_dir)?
        .iter()
        .map(|g| g.name)
        .collect::<Vec<_>>();

    info!("Found {} games\n{}", games.len(), games.join("\n"));

    Ok(())
}

const HD_GAME: &str = "_hd";
const ADDON_GAME: &str = "_addon";
const DLLS_DIR: &str = "dlls";
const CL_DLLS_DIR: &str = "cl_dlls";

pub struct DirType {
    pub name: String,
    pub has_hd: bool,
    pub has_addon: bool,
}

pub fn game_dir_types<P>(dir: P) -> Result<Vec<DirType>>
where
    P: AsRef<Path>,
{
    let dir = dir.as_ref();

    let mut types = Vec::new();

    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory")?;
        let path = entry.path();

        if path.is_dir() {
            let file_name = path
                .file_name()
                .context("Failed to get file name")?
                .to_string_lossy();

            let dir_type = types
                .iter_mut()
                .find(|t: &&mut DirType| t.name == file_name);
            let dir_type = match dir_type {
                Some(dir_type) => dir_type,
                None => {
                    if path.join(DLLS_DIR).is_dir() || path.join(CL_DLLS_DIR).is_dir() {
                        types.push(DirType {
                            name: file_name.to_string(),
                            has_hd: false,
                            has_addon: false,
                        });
                        types.last_mut().unwrap()
                    } else {
                        continue;
                    }
                }
            };

            if file_name.ends_with(HD_GAME) {
                dir_type.has_hd = true;
            } else if file_name.ends_with(ADDON_GAME) {
                dir_type.has_addon = true;
            }
        }
    }

    Ok(types)
}
