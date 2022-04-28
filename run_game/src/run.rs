use std::env;

use anyhow::{Result, Context};
use hltas_manager::cfg::{cfg_file_name, Cfg};
use log::info;

pub fn run() -> Result<()> {
    info!("Loading config...");
    let current_dir = env::current_dir().context("Failed to get current directory")?;
    let root_dir = current_dir
        .parent()
        .context("Failed to get root directory")?
        .parent()
        .context("Failed to get root directory")?;

    let cfg_dir = root_dir.join(cfg_file_name());
    let cfg = Cfg::load_from_path(cfg_dir).context("Failed to load cfg")?;

    todo!()
}
