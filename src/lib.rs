pub mod cfg;
pub mod helper;

use std::fs::{self};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use helper::root_dir;

use crate::cfg::Cfg;

pub const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Installs the required files for the tool.
    ///
    /// - Can be used again to verify if the files are already installed.
    /// - Requires the 'steamapi.dll' file to be the default dll as when you installed the game.
    /// - This command will download the required files as well as creating files for the tool to use.
    /// - Needs to be run before the tool can be used.
    // TODO verify if all steam_api.dll hash are the same, then remove the first doc comment line
    Install {
        #[clap(long)]
        projects_dir_name: Option<String>,
    },
    /// Create a new project.
    ///
    /// - The project is created in the 'tas' directory, but can be changed through the config file.
    New {
        #[clap(long, short = 'n')]
        project_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        // TODO depends on if game_name is set
        #[clap(long, short)]
        copy_game_dir_for_sim_client: bool,
        #[clap(long, short)]
        copy_cfgs: bool,
    },
    /// Initializes a new project in an existing directory.
    ///
    /// - This is the same as 'new' but it uses an existing directory.
    Init {
        #[clap(long, short = 'n')]
        folder_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        // TODO depends on if game_name is set
        #[clap(long, short)]
        copy_game_dir_for_sim_client: bool,
    },
    /// Lists all available games.
    ///
    /// - Lists all games installed in the 'Half-Life' directory.
    /// - A game is usually all directories in the Half-Life directory.
    /// - Able to set exclusions in the config file.
    // TODO auto detect if its a game or unrelated dir
    Games,
    /// Generates the .cfg files for TASing.
    ///
    /// - Creates the cfg files 'hltas.cfg', 'ingame.cfg', 'record.cfg', 'editor.cfg' and 'cam.cfg'.
    /// - These files will create keybinds for you to use while TASing, unless the 'minimum_cfgs' flag is set.
    /// - Read the comment in the config file for more information on what each one does.
    /// - If the files are already present in 'cfgs/' they will be used instead of creating new ones.
    GenCfgs {
        #[clap(long)]
        minimum_cfgs: bool,
        #[clap(long)]
        no_userconfig_change: bool,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    match &cli.command {
        Commands::Install { projects_dir_name } => {
            // verifying if the half-life directory exists
            let half_life_dir = helper::half_life_dir()?;
            let root_dir = root_dir()?;

            // copy the simulator client steam_api.dll (sim.dll)
            // TODO thread for this and the other stuff?
            let base_sim_client_dll_path = root_dir.join("sim.dll");

            if base_sim_client_dll_path.is_file() {
                let sim_client_dll_path = half_life_dir.join("sim.dll");

                fs::copy(base_sim_client_dll_path, sim_client_dll_path)
                    .context("Failed to copy sim.dll")?;
            } else {
                bail!("sim.dll not found in the root directory");
            }

            // TODO check other verifications

            // config path
            let config_path = helper::cfg_dir()?;

            if config_path.is_file() {
                println!("Config file already exists, skipping config creation.");
            } else {
                // create the config file
                Cfg::save_default_to_path(&config_path)?;
            }

            // load config
            let mut cfg = Cfg::load_from_path(&config_path)?;

            // save override
            if let Some(projects_dir_name) = projects_dir_name {
                cfg.project_dir_name = projects_dir_name.to_owned();

                // save config
                cfg.save_to_path(&config_path)?;
            }
            let cfg = cfg;

            // create project dir if it doesn't exist
            let projects_dir = root_dir.join(&cfg.project_dir_name);

            if !projects_dir.is_dir() {
                fs::create_dir(&projects_dir).context("Failed to create projects dir")?;
            }

            // copy default steam_api.dll as reset.dll
            // TODO steam_api verification?
            let steamapi_dll_path = half_life_dir.join("steam_api.dll");
            let reset_dll_path = half_life_dir.join("reset.dll");

            if steamapi_dll_path.is_file() {
                fs::copy(steamapi_dll_path, reset_dll_path)
                    .context("Failed to copy steam_api.dll to reset.dll")?;
            } else {
                bail!("steam_api.dll not found in the Half-Life directory");
            }
        }
        Commands::New {
            project_name,
            game_name,
            copy_game_dir_for_sim_client,
            copy_cfgs,
        } => todo!(),
        Commands::Init {
            folder_name,
            game_name,
            copy_game_dir_for_sim_client,
        } => todo!(),
        Commands::Games => todo!(),
        Commands::GenCfgs {
            minimum_cfgs,
            no_userconfig_change,
        } => todo!(),
    }

    todo!()
}
