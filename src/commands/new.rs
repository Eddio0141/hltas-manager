use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process,
};

use anyhow::{bail, Context, Result};
use fs_extra::dir::CopyOptions;

use crate::{cfg::Cfg, files, helper, DEFAULT_GAME};

use super::games;

pub fn new(
    project_name: &str,
    game_name: &Option<String>,
    copy_game_dir_for_sim_client: bool,
    init_git: bool,
    no_init_git: bool,
) -> Result<()> {
    // TODO check requirements of command before running it
    // load config
    let cfg = helper::cfg_dir()?;
    let cfg = Cfg::load_from_path(cfg)?;

    // paths
    let root_dir = helper::root_dir()?;
    let project_dir = root_dir.join(&cfg.project_dir).join(project_name);
    let default_game = DEFAULT_GAME.to_string();
    let game_name_full = game_name.as_ref().unwrap_or(&default_game);
    let half_life_dir = &cfg.half_life_dir;
    let game_dir = half_life_dir.join(game_name_full);

    game_dir_validate(&cfg, &game_name_full)?;

    if project_dir.exists() {
        bail!("Project folder already exists\nHelp: Use 'init' to initialize a project in an existing folder.");
    } else {
        fs::create_dir(&project_dir).context("Failed to create project folder")?;
    }

    let init_git = {
        if init_git {
            true
        } else if no_init_git {
            false
        } else {
            cfg.init_git_on_project
        }
    };

    if init_git {
        set_up_git(&project_dir)?;
    }

    // link cfgs if needed to
    // if cfg.link_cfgs_to_new_game {
    //     files::hard_link_cfgs(&project_dir)?;
    // }

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
        if *game_name != DEFAULT_GAME {
            let client_dll_path = game_dir.join("cl_dlls").join("client.dll");

            // if it exists, copy to the secondary game instance folder, or generate toggle batch files
            if client_dll_path.is_file() {
                match &cfg.no_client_dll_dir {
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
                                fs_extra::dir::copy(&game_dir, &copy_path, &copy_options).context(
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
    files::write_hltas_linker(&project_dir, half_life_dir)?;

    // TODO
    // create run_game file

    Ok(())
}

fn game_dir_validate(cfg: &Cfg, game_name: &str) -> Result<()> {
    let half_life_dir = &cfg.half_life_dir;
    let game_dir = half_life_dir.join(game_name);

    // check if game is installed or not excluded
    let games = games::games_in_dir(half_life_dir)?;

    if cfg.ignore_games.iter().any(|g| games.contains(g)) {
        bail!("Can't create project for game that is ignored in the config");
    }

    // if game dir doesn't exist
    if !game_dir.is_dir() {
        bail!(
            "Game '{game_name}' not found in the {} directory",
            half_life_dir.file_name().unwrap().to_string_lossy()
        );
    }

    if !game_dir.is_dir() {
        bail!("Game directory not found");
    }

    Ok(())
}

fn set_up_git<P>(project_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let project_dir = project_dir.as_ref();

    if project_dir.join(".git").is_dir() {
        return Ok(());
    }

    process::Command::new("git")
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

    Ok(())
}
