use std::{
    env::current_dir,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use log::{debug, info};

use crate::{
    cfg::{self, Cfg},
    helper, project_toml,
};

pub fn link_hltas(keep_alive: bool) -> Result<()> {
    let current_dir = current_dir().context("Failed to get current directory")?;
    let project_toml_path = current_dir.join(project_toml::FILE_NAME);

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

    if keep_alive {
        loop {
            link_hltas_once(project_toml_path.is_file(), &current_dir, &cfg, true)?;
            std::thread::sleep(Duration::from_secs(1));
        }
    } else {
        link_hltas_once(project_toml_path.is_file(), current_dir, &cfg, false)?;
    }

    Ok(())
}

pub fn link_hltas_once<P: AsRef<Path>>(
    is_in_project_dir: bool,
    current_dir: P,
    cfg: &Cfg,
    silent: bool,
) -> Result<()> {
    let hltases_from_dir = |dir: &Path| -> Result<Vec<PathBuf>> {
        let mut hltases = Vec::new();

        for dir in dir
            .read_dir()
            .with_context(|| format!("Failed to read directory {}", dir.display()))?
        {
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

    let current_dir = current_dir.as_ref();
    let root_dir = if is_in_project_dir {
        let tas_dir = current_dir.parent().context("Failed to get root dir")?;
        tas_dir.parent().context("Failed to get root dir parent")?
    } else {
        current_dir
    };
    let half_life_dir = root_dir.join(&cfg.half_life_dir);

    let hltases = if is_in_project_dir {
        hltases_from_dir(current_dir)?
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

    debug!("HLTASes: {:?}", hltases);

    for hltas in hltases {
        // hard-link to main game
        if !silent {
            info!("Linking {}", hltas.display());
        }
        let game_dir_hltas = half_life_dir.join(hltas.file_name().unwrap());
        debug!(
            "Linking {} to {}",
            hltas.display(),
            game_dir_hltas.display()
        );
        helper::force_link(&hltas, &game_dir_hltas).context("Failed to hard link hltas")?;

        if let Some(second_game_dir) = &cfg.no_client_dll_dir {
            // hard-link to second game
            let game_dir_hltas = root_dir.join(second_game_dir.join(hltas.file_name().unwrap()));

            debug!(
                "Linking {} to {}",
                hltas.display(),
                game_dir_hltas.display()
            );
            helper::force_link(&hltas, &game_dir_hltas).context("Failed to hard link hltas")?;
        }
    }

    Ok(())
}
