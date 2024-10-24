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
            reset_cfgs,
        } => {
            install(install::Override {
                projects_dir,
                half_life_dir,
                minimum_cfgs: *minimum_cfgs,
                reset_cfgs,
            })?;
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
            script: run_script,
            params,
            r_input,
            game_override,
            optim_games,
            keep_alive,
        } => {
            run_game(
                RunGameMiscFlags { r_input: *r_input },
                RunGameFlags {
                    low: *low,
                    vanilla_game: *vanilla_game,
                    width: *width,
                    height: *height,
                    params,
                    game_override,
                    keep_alive: *keep_alive,
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
