pub mod cfg;
pub mod files;
pub mod helper;

use std::{
    fs::{self, File},
    io::Write,
    process::Command,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use fs_extra::dir::CopyOptions;
use helper::{half_life_dir, root_dir};

use crate::cfg::Cfg;

// TODO other os support

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
        #[clap(long, conflicts_with = "no-init-git")]
        init_git: bool,
        #[clap(long)]
        no_init_git: bool,
        #[clap(long, conflicts_with = "no-copy-hltas-cfgs")]
        copy_hltas_cfgs: bool,
        #[clap(long)]
        no_copy_hltas_cfgs: bool,
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
            init_git,
            no_init_git,
            copy_hltas_cfgs,
            no_copy_hltas_cfgs,
        } => {
            // TODO check requirements of command before running it
            // load config
            let cfg = helper::cfg_dir()?;
            let cfg = Cfg::load_from_path(cfg)?;

            // create project dir
            let root_dir = helper::root_dir()?;
            let project_dir = root_dir.join(cfg.project_dir_name).join(project_name);
            let game_name_full = game_name.as_ref().unwrap_or(&cfg.default_game);
            let half_life_dir = helper::half_life_dir()?;
            let game_dir = half_life_dir.join(game_name_full);

            if project_dir.exists() {
                bail!("Project folder already exists\nHelp: Use 'init' to initialize a project in an existing folder.");
            } else {
                fs::create_dir(&project_dir).context("Failed to create project folder")?;
            }

            let init_git = {
                if *init_git {
                    true
                } else if *no_init_git {
                    false
                } else {
                    cfg.init_git_on_project
                }
            };

            if init_git {
                Command::new("git")
                    .current_dir(&project_dir)
                    .arg("init")
                    .output()
                    .context("Failed to init git\nHelp: Use '--no-init-git' to skip git init\nNote: This process failing could be due to git not being installed")?;

                // add hardlink hook to .git/hooks/post-checkout
                let post_checkout_hook_path = project_dir.join(".git/hooks/post-checkout");

                files::write_hard_link_shell_hook(post_checkout_hook_path)?;

                // create .gitignore file
                let gitignore_path = project_dir.join(".gitignore");
                let gitignore = "*.bat";
                let gitignore = format!("\n{gitignore}");

                let mut gitignore_file = if gitignore_path.is_file() {
                    // append to .gitignore

                    fs::OpenOptions::new()
                        .append(true)
                        .open(gitignore_path)
                        .context("Failed to open .gitignore")?
                } else {
                    File::create(gitignore_path).context("Failed to create .gitignore")?
                };

                gitignore_file
                    .write_all(gitignore.as_bytes())
                    .context("Failed to write to .gitignore")?;
            }

            // copy cfgs if needed to
            if cfg.copy_cfgs_to_new_game {
                files::write_cfgs(&project_dir)?;
            }

            // TODO do this process in the install command, since the whole dir is copied
            // let client_dll_path = game_dir.join("cl_dlls").join("client.dll");

            // // only if client.dll exists, we copy the game dir
            // if *copy_game_dir_for_sim_client && client_dll_path.is_file() {
            //     // copy the game dir for the main client
            //     let copy_path = game_dir
            //         .parent()
            //         .unwrap()
            //         .join(format!("{game_name}_{}", &cfg.no_client_dll_dir_name));

            //     if game_dir.is_dir() {
            //         fs::copy(&game_dir, &copy_path).context("Failed to copy game dir")?;

            //         // remove the client.dll
            //         let copied_client_dll = copy_path.join("cl_dlls").join("client.dll");

            //         fs::remove_file(copied_client_dll).context("Failed to remove client.dll")?;
            //     } else {
            //         bail!("Game directory not found");
            //     }
            // } else {
            //     // generate toggle sim client batch files
            //     files::write_toggle_sim_client(&project_dir)?;
            // }

            // if this is a non-default game, check if cl_dlls/client.dll exists
            if let Some(game_name) = game_name {
                if *game_name != cfg.default_game {
                    let client_dll_path = game_dir.join("cl_dlls").join("client.dll");

                    // if it exists, copy to the secondary game instance folder, or generate toggle batch files
                    if client_dll_path.is_file() {
                        match &cfg.no_client_dll_dir_name {
                            Some(no_client_dll_dir_name) => {
                                if no_client_dll_dir_name.is_dir() {
                                    // copy the game dir
                                    let copy_path =
                                        no_client_dll_dir_name.join("Half-Life").join(game_name);

                                    if !copy_path.is_dir() {
                                        let copy_options = CopyOptions {
                                            overwrite: true,
                                            ..Default::default()
                                        };
                                        fs_extra::dir::copy(&game_dir, &copy_path, &copy_options)
                                            .context(
                                            "Failed to copy game dir to secondary game instance",
                                        )?;
                                    }
                                } else {
                                    bail!("Failed to find secondary game instance folder");
                                }
                            }
                            None => {
                                // generate toggle sim client batch files
                                files::write_toggle_vanilla_game(&project_dir, &game_dir)?;
                            }
                        }
                    }
                }
            }

            // create linker batch file
            files::write_hltas_linker(&project_dir, &half_life_dir)?;

            // TODO
            // create run_game file

            // copy cfg files on game if it doesn't exist and cfgs needs copying
            let copy_cfg_files = {
                if *copy_hltas_cfgs {
                    true
                } else if *no_copy_hltas_cfgs {
                    false
                } else {
                    cfg.copy_cfgs_to_new_game
                }
            };

            todo!()
        }
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
