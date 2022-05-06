pub mod games;
pub mod install;
pub mod link;
pub mod project;
pub mod run_game;
pub mod sync_saves;

use anyhow::Result;
use log::info;

use crate::{
    cfg::Cfg,
    cli::{Cli, Commands},
    commands::sync_saves::sync_saves,
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
#[cfg(debug_assertions)]
use log::debug;

pub fn run(cli: Cli) -> Result<()> {
    #[cfg(debug_assertions)]
    debug!("running app with args: {:#?}", &cli);

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
            use_batch_scripts,
        } => {
            new(
                project_name,
                game_name,
                *init_git,
                *no_init_git,
                *use_batch_scripts,
            )?;
            info!("Created project!");
        }
        Commands::Init {
            folder_name,
            game_name,
            init_git,
            no_init_git,
            use_batch_scripts,
        } => {
            init(
                folder_name,
                game_name,
                *init_git,
                *no_init_git,
                *use_batch_scripts,
            )?;
            info!("Initialized project!");
        }
        Commands::Games => {
            // load config
            let cfg = helper::cfg_dir()?;
            let cfg = Cfg::load(cfg)?;

            let root = helper::exe_dir()?;
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
        Commands::SyncSaves => {
            sync_saves()?;
            info!("Synced saves!");
        }
    }

    Ok(())
}
