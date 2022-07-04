use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::DEFAULT_GAME;

pub const FILE_NAME: &str = "project.toml";

#[derive(Serialize, Deserialize)]
pub struct ProjectToml {
    pub game: String,
    // TODO command line options for project
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
        let project = fs::read_to_string(path).context("Failed to read project config file")?;
        let project: ProjectToml =
            toml::from_str(&project).context("Failed to parse project config file")?;

        Ok(project)
    }

    pub fn save_to_path<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let contents = toml::to_string(self)?;
        fs::write(path, contents).context("Could not write to config file")?;

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
