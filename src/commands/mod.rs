pub mod games;
pub mod install;
pub mod new;

use anyhow::Result;
use log::info;

use crate::{
    cfg::Cfg,
    cli::{Cli, Commands},
    helper,
};

use self::{games::games, install::install, new::new};

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
            copy_game_dir_for_sim_client,
            init_git,
            no_init_git,
        } => {
            new(
                project_name,
                game_name,
                *copy_game_dir_for_sim_client,
                *init_git,
                *no_init_git,
            )?;
        }
        Commands::Init {
            folder_name,
            game_name,
            copy_game_dir_for_sim_client,
        } => todo!(),
        Commands::Games => {
            // load config
            let cfg = helper::cfg_dir()?;
            let cfg = Cfg::load_from_path(cfg)?;

            let root = helper::root_dir()?;
            let half_life_dir = root.join(&cfg.half_life_dir);

            games(half_life_dir)?;
        }
    }

    Ok(())
}
