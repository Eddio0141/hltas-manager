pub mod games;
pub mod install;
pub mod link;
pub mod project;
pub mod run_game;

use anyhow::Result;
use log::info;

use crate::{
    cfg::Cfg,
    cli::{Cli, Commands},
    helper::{self},
};

use self::{
    games::games,
    install::install,
    link::link,
    project::init,
    project::new,
    run_game::{run_game, RunGameFlags},
};

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
            let cfg = Cfg::load(cfg)?;

            let root = helper::root_dir()?;
            let half_life_dir = root.join(&cfg.half_life_dir);

            games(half_life_dir)?;
        }
        Commands::RunGame {
            sim,
            low,
            vanilla_game,
            record,
            width,
            height,
            no_bxt,
            run_script,
            params,
            r_input,
            no_tas_view,
            game_override,
        } => {
            run_game(
                RunGameFlags {
                    sim: *sim,
                    low: *low,
                    vanilla_game: *vanilla_game,
                    record: *record,
                    no_bxt: *no_bxt,
                    r_input: *r_input,
                    no_tas_view: *no_tas_view,
                },
                *width,
                *height,
                run_script,
                params,
                game_override,
            )?;
        }
        Commands::Link => {
            link()?;
            info!("Linked hltases!");
        }
    }

    Ok(())
}
