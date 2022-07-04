use std::{fs, path::Path};

use anyhow::{Context, Result};
use log::info;

pub fn games<P>(half_life_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let games = game_dir_types(half_life_dir)?
        .into_iter()
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

impl DirType {
    pub fn dir_names(&self) -> Vec<String> {
        let mut dir_names = Vec::with_capacity(3);

        dir_names.push(self.name.clone());

        if self.has_hd {
            dir_names.push(format!("{}{}", &self.name, HD_GAME));
        }
        if self.has_addon {
            dir_names.push(format!("{}{}", &self.name, ADDON_GAME));
        }

        dir_names
    }
}

pub fn game_dir_types<P>(dir: P) -> Result<Vec<DirType>>
where
    P: AsRef<Path>,
{
    let dir = dir.as_ref();

    let mut types = Vec::new();

    for entry in
        fs::read_dir(dir).with_context(|| format!("Failed to read directory {}", dir.display()))?
    {
        let entry = entry.with_context(|| format!("Failed to read directory {}", dir.display()))?;
        let path = entry.path();

        if path.is_dir() {
            let file_name = path
                .file_name()
                .context("Failed to get file name")?
                .to_string_lossy()
                .to_string();

            let is_game = path.join(DLLS_DIR).is_dir() || path.join(CL_DLLS_DIR).is_dir();
            let is_hd = file_name.ends_with(HD_GAME);
            let is_addon = file_name.ends_with(ADDON_GAME);

            let file_name = {
                if is_hd {
                    file_name.trim_end_matches(HD_GAME)
                } else if is_addon {
                    file_name.trim_end_matches(ADDON_GAME)
                } else {
                    &file_name
                }
            };

            let dir_type = types
                .iter_mut()
                .find(|t: &&mut DirType| t.name == file_name);

            if !is_game && !is_hd && !is_addon {
                continue;
            } else if let Some(dir_type) = dir_type {
                if is_hd {
                    dir_type.has_hd = true;
                } else if is_addon {
                    dir_type.has_addon = true;
                }
            } else {
                types.push(DirType {
                    name: file_name.to_string(),
                    has_hd: is_hd,
                    has_addon: is_addon,
                });
            }
        }
    }

    Ok(types)
}
