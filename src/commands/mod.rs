pub mod games;
pub mod install;
pub mod project;

use anyhow::Result;
use log::info;

use crate::{
    cfg::Cfg,
    cli::{Cli, Commands},
    helper::{self, root_dir},
};

use self::{games::games, install::install, project::init, project::new};

pub fn run(cli: Cli) -> Result<()> {
    match &cli.command {
        Commands::Install {
            projects_dir,
            half_life_dir,
            minimum_cfgs,
        } => {
            install(projects_dir, half_life_dir, *minimum_cfgs)?;
            info!("Installed!");
        }
        Commands::New {
            project_name,
            game_name,
            init_git,
            no_init_git,
        } => {
            new(project_name, game_name, *init_git, *no_init_git)?;
            info!("Created project!");
        }
        Commands::Init {
            folder_name,
            game_name,
            init_git,
            no_init_git,
        } => {
            init(folder_name, game_name, *init_git, *no_init_git)?;
            info!("Initialized project!");
        }
        Commands::Games => {
            // load config
            let cfg = helper::cfg_dir()?;
            let cfg = Cfg::load_from_path(cfg)?;

            let root = helper::root_dir()?;
            let half_life_dir = root.join(&cfg.half_life_dir);

            games(half_life_dir)?;
        }
        Commands::RunGame {
            sim,
            low,
            no_vanilla,
            record,
            width,
            height,
            no_bxt,
            run_script,
            params,
            no_r_input,
        } => {
            // let root_dir = root_dir()?;

            // info!("Loading config...");
            // let cfg_dir = root_dir.join(cfg_file_name());
            // let cfg = Cfg::load_from_path(cfg_dir).context("Failed to load cfg")?;

            // let half_life_dir = root_dir.join(&cfg.half_life_dir);
            // let hl_exe = half_life_dir.join("hl.exe");

            // let r_input_path = root_dir.join("RInput").join("RInput.exe");

            todo!()
        }
    }

    Ok(())
}
