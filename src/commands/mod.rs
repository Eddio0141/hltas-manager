pub mod games;
pub mod install;
pub mod link_hltas;
pub mod project;
pub mod run_game;
pub mod sync;
pub mod sync_saves;

use anyhow::Result;
use log::info;

use crate::{
    cfg::Cfg,
    cli::{Cli, Commands},
    commands::run_game::RunGameFlags,
    helper::{self},
};

use self::{
    games::games, install::install, link_hltas::link_hltas, project::init, project::new,
    run_game::*, sync::sync, sync_saves::sync_saves,
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
            optim_games,
        } => {
            run_game(
                RunGameMiscFlags {
                    r_input: *r_input,
                    no_tas_view: *no_tas_view,
                },
                RunGameFlags {
                    low: *low,
                    vanilla_game: *vanilla_game,
                    width: *width,
                    height: *height,
                    params,
                    game_override,
                },
                RunGameBxtFlags {
                    run_script,
                    optim_games,
                    sim: *sim,
                    record: *record,
                    no_bxt: *no_bxt,
                },
            )?;
        }
        Commands::LinkHLTAS { keep_alive } => {
            link_hltas(*keep_alive)?;
            info!("Linked hltases!");
        }
        Commands::SyncSaves { keep_alive } => {
            sync_saves(*keep_alive)?;
        }
        Commands::Sync => {
            sync()?;
        }
    }

    Ok(())
}
