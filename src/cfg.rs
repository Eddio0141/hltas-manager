use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Cfg {
    pub init_git_on_project: bool,
    pub project_dir_name: String,
    pub ignore_game_dirs: Vec<String>,
    pub copy_cfgs_to_userconfig_on_project_game: bool,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            init_git_on_project: true,
            project_dir_name: "tas".to_string(),
            ignore_game_dirs: vec![".bxt-ipc".to_string()],
            copy_cfgs_to_userconfig_on_project_game: false,
        }
    }
}

impl Cfg {
    pub fn load_from_path<P>(path: P) -> Result<Cfg>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path).context("Failed to open config file")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .context("Failed to read config file")?;
        let cfg: Cfg = toml::from_str(&contents).context("Failed to parse config file")?;

        Ok(cfg)
    }

    pub fn save_to_path<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let contents = toml::to_string(self)?;

        let mut file = File::create(path).context("Could not create config file")?;
        file.write_all(contents.as_bytes())
            .context("Could not write to config file")?;

        Ok(())
    }

    pub fn save_default_to_path<P>(path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let cfg = Cfg::default();
        cfg.save_to_path(path)
    }
}

pub fn cfg_file_name() -> String {
    format!("{}.toml", crate::NAME)
}
