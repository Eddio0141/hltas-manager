use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
};

use anyhow::{bail, Context, Result};
use fs_extra::dir::CopyOptions;
use log::info;
use sha2::{Digest, Sha256};

use crate::{
    cfg::Cfg,
    files, helper,
    project_toml::{self, ProjectToml},
    DEFAULT_GAME,
};

use super::games;

use lazy_static::lazy_static;

pub fn new(
    project_name: &str,
    game_name: &Option<String>,
    init_git: bool,
    no_init_git: bool,
) -> Result<()> {
    let cfg = helper::cfg_dir()?;
    let cfg = Cfg::load(cfg)?;
    let root_dir = helper::exe_dir()?;
    let project_dir = root_dir.join(&cfg.project_dir).join(project_name);

    info!("Creating project directory...");
    if project_dir.exists() {
        bail!("Project folder already exists\nHelp: Use 'init' to initialize a project in an existing folder.");
    } else {
        fs::create_dir(&project_dir).context("Failed to create project folder")?;
    }

    init_project(project_dir, game_name, init_git, no_init_git)
}

pub fn init(
    project_name: &str,
    game_name: &Option<String>,
    init_git: bool,
    no_init_git: bool,
) -> Result<()> {
    let cfg = helper::cfg_dir()?;
    let cfg = Cfg::load(cfg)?;
    let root_dir = helper::exe_dir()?;
    let project_dir = root_dir.join(&cfg.project_dir).join(project_name);

    // check if project folder exists
    if !project_dir.is_dir() {
        bail!("Project folder does not exist, Help: Use 'new' to create a new project.");
    }

    init_project(project_dir, game_name, init_git, no_init_git)
}

fn init_project<P>(
    project_dir: P,
    game_name: &Option<String>,
    init_git: bool,
    no_init_git: bool,
) -> Result<()>
where
    P: AsRef<Path>,
{
    // load config
    info!("Loading config");
    let cfg = helper::cfg_dir()?;
    let cfg = Cfg::load(cfg)?;

    // paths
    let root_dir = helper::exe_dir()?;
    let default_game = DEFAULT_GAME.to_string();
    let game_name_full = game_name.as_ref().unwrap_or(&default_game);
    let half_life_dir = &cfg.half_life_dir;
    let game_dir = root_dir.join(half_life_dir.join(game_name_full));

    // validate if second client exists
    let second_game_dir = validate_second_client(&cfg, &root_dir, game_name_full)?;

    // validate if game dir exists
    game_dir_validate(&cfg, game_name_full)?;

    // check on project.toml
    let project_toml = project_dir.as_ref().join(project_toml::FILE_NAME);

    if !project_toml.is_file() {
        info!("Creating project.toml");

        let project = ProjectToml {
            game: game_name_full.to_string(),
        };

        project.save_to_path(project_toml)?;
    }

    // copy game dir
    // will only copy if it doesn't exist
    if let Some(second_game_dir) = second_game_dir.as_ref() {
        copy_game_dir(second_game_dir, &game_dir)?;
    }

    // override userconfig.cfg
    override_userconfig(&game_dir, &second_game_dir)?;

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
        set_up_git(&project_dir, &cfg)?;
    }

    Ok(())
}

fn override_userconfig<P, P2>(game_dir: P, second_game_dir: &Option<P2>) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let config = "exec hltas.cfg\nloadtas2";

    let userconfig_path = "userconfig.cfg";
    let game_dir = game_dir.as_ref();

    info!("Writing userconfig.cfg for game directory...");
    let game_dir_userconfig = game_dir.join(userconfig_path);
    if game_dir_userconfig.is_file() {
        OpenOptions::new()
            .append(true)
            .open(game_dir_userconfig)?
            .write_all(format!("\n{config}").as_bytes())
            .context("Failed to write to userconfig.cfg")?;
    } else {
        fs::write(game_dir_userconfig, config).context("Failed to write to userconfig.cfg")?;
    }

    if let Some(second_game_dir) = second_game_dir {
        let second_game_dir = second_game_dir.as_ref();
        let second_game_dir_userconfig = second_game_dir.join(userconfig_path);

        info!("Writing userconfig.cfg for second game directory...");
        if second_game_dir_userconfig.is_file() {
            OpenOptions::new()
                .append(true)
                .open(second_game_dir_userconfig)?
                .write_all(format!("\n{config}").as_bytes())
                .context("Failed to write to userconfig.cfg")?;
        } else {
            fs::write(second_game_dir_userconfig, config)
                .context("Failed to write to userconfig.cfg")?;
        }
    }

    Ok(())
}

fn copy_game_dir<P, P2>(second_game_dir: P, game_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let second_game_dir = second_game_dir.as_ref();
    info!("Copying game directory to second client...");
    let copy_options = CopyOptions {
        skip_exist: true,
        copy_inside: true,
        ..Default::default()
    };

    fs_extra::dir::copy(&game_dir, second_game_dir, &copy_options).with_context(|| {
        format!(
            "Failed to copy game dir from {} to {}",
            game_dir.as_ref().display(),
            second_game_dir.display()
        )
    })?;

    // remove client.dll if it exists unless default game
    if let Some(game_name) = game_dir.as_ref().file_name() {
        if game_name.to_string_lossy() == DEFAULT_GAME {
            info!("Copying game is the default game, skipping removal of client.dll");
        } else {
            info!("Removing client.dll from second client...");
            let second_game_client_dll = second_game_dir.join("cl_dlls").join("client.dll");
            if second_game_client_dll.is_file() {
                fs::remove_file(second_game_client_dll).context("Failed to remove client.dll")?;
            }
        }
    }

    Ok(())
}

fn validate_second_client<P>(
    cfg: &Cfg,
    root_dir: P,
    game_name_full: &str,
) -> Result<Option<PathBuf>>
where
    P: AsRef<Path>,
{
    let root_dir = root_dir.as_ref();
    let second_client_dir = match &cfg.no_client_dll_dir {
        Some(no_client_dll_dir) => {
            let second_client_dir = root_dir.join(no_client_dll_dir);

            if !second_client_dir.is_dir() {
                bail!("Second client directory does not exist\nHelp: Run 'install' command first");
            }

            Some(second_client_dir.join(game_name_full))
        }
        None => None,
    };

    Ok(second_client_dir)
}

fn game_dir_validate(cfg: &Cfg, game_name: &str) -> Result<()> {
    let half_life_dir = &cfg.half_life_dir;
    let game_dir = half_life_dir.join(game_name);

    // check if game is installed or not excluded
    let games = games::game_dir_types(half_life_dir)?;

    if cfg
        .ignore_games
        .iter()
        .any(|g| games.iter().any(|game| &game.name == g))
    {
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

const GIT_IGNORE: &str = include_str!("../../resource/git_ignores/project");
lazy_static! {
    static ref GIT_IGNORE_SHA_256: Vec<u8> = {
        let mut hasher = Sha256::new();
        hasher.update(GIT_IGNORE);
        hasher.finalize().to_vec()
    };
}

fn set_up_git<P>(project_dir: P, cfg: &Cfg) -> Result<()>
where
    P: AsRef<Path>,
{
    let project_dir = project_dir.as_ref();

    if project_dir.join(".git").is_dir() {
        info!("Project already has a git repository, skipping git init");
    } else {
        info!("Setting up git repository...");
        process::Command::new("git")
        .current_dir(project_dir)
        .arg("init")
        .output()
        .context("Failed to init git\nHelp: Use '--no-init-git' to skip git init\nNote: This process failing could be due to git not being installed")?;
    }

    // add hardlink hook to .git/hooks/post-checkout
    let post_checkout_hook_path = project_dir.join(".git/hooks/post-checkout");

    files::write_hard_link_shell_hook(post_checkout_hook_path, cfg)?;

    // create .gitignore file
    let gitignore_path = project_dir.join(".gitignore");

    if gitignore_path.is_file() {
        // append to .gitignore if sha256 isn't the same
        if helper::sha_256_file(&gitignore_path).context("Failed to get sha256 of .gitignore")?
            != *GIT_IGNORE_SHA_256
        {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&gitignore_path)
                .context("Failed to open .gitignore")?;

            file.write_all(GIT_IGNORE.as_bytes())
                .context("Failed to write to .gitignore")?;

            let gitignore = format!("\n{GIT_IGNORE}");

            file.write_all(gitignore.as_bytes())
                .context("Failed to write to .gitignore")?;
        }
    } else {
        info!("Creating .gitignore file...");
        let mut file = File::create(gitignore_path).context("Failed to create .gitignore")?;
        file.write_all(GIT_IGNORE.as_bytes())
            .context("Failed to write to .gitignore")?;
    }

    Ok(())
}
