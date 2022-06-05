use std::{env::current_dir, path::Path, thread, time::Duration};

use log::{debug, info};

use crate::{
    cfg::{self, Cfg},
    commands::{link_hltas::link_hltas_once, sync_saves::sync_saves_once},
    project_toml,
};
use anyhow::{Context, Result};

pub fn sync() -> Result<()> {
    info!("Starting sync...");

    let current_dir = current_dir().context("Failed to get current directory")?;
    let project_toml_path = current_dir.join(project_toml::FILE_NAME);
    let save = Path::new("SAVE");
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

    debug!(
        "project_toml_path: {}, current_dir: {}, root_dir: {}",
        project_toml_path.display(),
        current_dir.display(),
        root_dir.display(),
    );

    loop {
        link_hltas_once(project_toml_path.is_file(), &current_dir, &cfg, true)?;

        if let Some(no_client_dll_dir) = &cfg.no_client_dll_dir {
            sync_saves_once(
                save,
                root_dir.join(&cfg.half_life_dir),
                root_dir.join(no_client_dll_dir),
            )?;
        }

        thread::sleep(Duration::from_secs(1));
    }
}
