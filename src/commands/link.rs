use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::info;

use crate::{
    cfg::{self, Cfg},
    project_toml,
};

pub fn link() -> Result<()> {
    let current_dir = current_dir().context("Failed to get current directory")?;
    let project_toml_path = current_dir.join(project_toml::FILE_NAME);

    let hltases_from_dir = |dir: &Path| -> Result<Vec<PathBuf>> {
        let mut hltases = Vec::new();

        for dir in dir.read_dir().context("Failed to read directory")? {
            let dir = dir.context("Failed to read dir")?;
            let path = dir.path();

            if let Some(extension) = path.extension() {
                if extension == "hltas" {
                    hltases.push(path);
                }
            }
        }

        Ok(hltases)
    };

    let root_dir = if project_toml_path.is_file() {
        let project_dir = project_toml_path
            .parent()
            .context("Failed to get project toml parent")?;
        let tas_dir = project_dir.parent().context("Failed to get root dir")?;
        tas_dir.parent().context("Failed to get root dir parent")?
    } else {
        &current_dir
    };

    info!("Loading config...");
    let cfg_path = root_dir.join(cfg::cfg_file_name());
    let cfg = Cfg::load(cfg_path).context("Failed to load cfg")?;

    let hltases = if project_toml_path.is_file() {
        hltases_from_dir(&current_dir)?
    } else {
        let projects = current_dir.join(&cfg.project_dir);

        let mut hltases = Vec::new();

        for project in projects.read_dir().context("Failed to read project dir")? {
            let project = project.context("Failed to read project file")?;
            let path = project.path();

            hltases.extend(hltases_from_dir(&path)?);
        }

        hltases
    };

    let half_life_dir = root_dir.join(&cfg.half_life_dir);

    for hltas in hltases {
        // hard-link to main game
        info!("Linking {}", hltas.display());
        let game_dir_hltas = half_life_dir.join(hltas.file_name().unwrap());
        if game_dir_hltas.is_file() {
            info!("File already exists, removing");
            fs::remove_file(&game_dir_hltas).context("Failed to remove hltas")?;
        }
        fs::hard_link(&hltas, &game_dir_hltas).context("Failed to hard link hltas")?;

        if let Some(second_game_dir) = &cfg.no_client_dll_dir {
            // hard-link to second game
            let game_dir_hltas = root_dir.join(second_game_dir.join(hltas.file_name().unwrap()));

            if game_dir_hltas.is_file() {
                info!("File already exists, removing");
                fs::remove_file(&game_dir_hltas).context("Failed to remove hltas")?;
            }
            fs::hard_link(&hltas, &game_dir_hltas).context("Failed to hard link hltas")?;
        }
    }

    Ok(())
}
