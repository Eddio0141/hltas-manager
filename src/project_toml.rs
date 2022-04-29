use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::DEFAULT_GAME;

pub const FILE_NAME: &str = "project.toml";

#[derive(Serialize, Deserialize)]
pub struct ProjectToml {
    pub game: String,
}

impl Default for ProjectToml {
    fn default() -> Self {
        Self {
            game: DEFAULT_GAME.to_string(),
        }
    }
}

impl ProjectToml {
    pub fn load_from_path<P>(path: P) -> Result<ProjectToml>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path).context("Failed to open project config file")?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .context("Failed to read project config file")?;
        let project: ProjectToml =
            toml::from_str(&contents).context("Failed to parse project config file")?;

        Ok(project)
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
        let project = ProjectToml::default();
        project.save_to_path(path)
    }
}
