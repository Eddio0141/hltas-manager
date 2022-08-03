use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use fs_extra::dir::CopyOptions;
use log::{info, warn};

use crate::{
    cfg::Cfg,
    commands::games::game_dir_types,
    files,
    helper::{self, exe_dir, force_link},
    DEFAULT_GAME,
};

pub const STEAM_API_DLL_HASH: &[u8] = &[
    0x8c, 0x07, 0x3e, 0x0d, 0x2c, 0xa3, 0x9d, 0x1e, 0x98, 0x6b, 0xec, 0x34, 0x8f, 0x98, 0x83, 0x03,
    0x35, 0x7b, 0xe5, 0xc4, 0x95, 0xcc, 0xf6, 0xe0, 0x41, 0x58, 0x02, 0xb8, 0x6e, 0xae, 0x35, 0x34,
];

pub struct Override<'a> {
    pub projects_dir: &'a Option<PathBuf>,
    pub half_life_dir: &'a Option<PathBuf>,
    pub minimum_cfgs: bool,
    pub reset_cfgs: &'a Option<Vec<String>>,
}

pub fn install(override_: Override) -> Result<()> {
    let Override {
        projects_dir,
        half_life_dir,
        minimum_cfgs,
        reset_cfgs,
    } = override_;

    // config
    info!("Loading manager config");
    let config_path = helper::cfg_dir()?;
    let cfg = cfg_file_set_up(
        config_path,
        ProjectCfgOverrides {
            projects_dir_name: projects_dir.as_deref(),
            half_life_dir: half_life_dir.as_deref(),
        },
    )?;

    // paths
    let root_dir = exe_dir()?;
    let hl_dir = root_dir.join(&cfg.half_life_dir);
    let projects_dir = root_dir.join(&cfg.project_dir);
    let sim_dll = "_sim.dll";
    let reset_dll = "_reset.dll";
    let steam_api_dll = "steam_api.dll";
    let base_sim_client_dll_path = root_dir.join(sim_dll);
    let steam_api_dll_path = hl_dir.join(steam_api_dll);
    let reset_dll_path = hl_dir.join(reset_dll);

    // verifying if the half-life directory exists
    info!("Verifying files");
    if !hl_dir.is_dir() {
        bail!("Half-life directory does not exist, possible that you don't have the manager in a GoldSrc Package folder");
    }
    // verifying that _sim.dll exists in root dir
    if !base_sim_client_dll_path.is_file() {
        bail!("_sim.dll does not exist in the root directory");
    }
    // verifying if steam_api.dll exists
    if !steam_api_dll_path.is_file() {
        bail!("steam_api.dll does not exist in the Half-Life directory");
    }
    // verify if steam_api.dll hash is matching
    let steam_api_dll_hash = helper::sha_256_file(&steam_api_dll_path)?;

    if STEAM_API_DLL_HASH != steam_api_dll_hash.as_slice() {
        // check for _reset.dll hash
        if reset_dll_path.is_file() {
            // check if _reset.dll exists and that hash is matching
            let reset_dll_hash = helper::sha_256_file(&reset_dll_path)?;

            if STEAM_API_DLL_HASH != reset_dll_hash.as_slice() {
                bail!("_reset.dll hash is not matching default steam_api.dll hash");
            }
        } else {
            warn!("steam_api.dll hash is not matching default steam_api.dll hash and {reset_dll} does not exist, proceeding without simulator client dll");
        }
    }

    // create projects dir if it doesn't exist
    if !projects_dir.is_dir() {
        info!("Creating projects directory");
        fs::create_dir_all(&projects_dir).context("Failed to create projects directory")?;
    }

    // copy half life directory if needs to be copied
    if let Some(no_client_dll_dir) = &cfg.no_client_dll_dir {
        let no_client_dll_dir = root_dir.join(no_client_dll_dir);

        let mut copy_paths = Vec::new();

        let game_dirs = game_dir_types(&hl_dir)?;

        let dirs = hl_dir
            .read_dir()
            .context("Failed to read Half-Life directory")?;
        for entry in dirs {
            // check if path is a game directory
            let entry = entry.context("Failed to read Half-Life directory")?;
            let path = entry.path();

            // we exclude game dir from being copied unless its the default game
            if let Some(path_name) = path.file_name() {
                let path_name = path_name.to_string_lossy().to_string();

                if path_name == DEFAULT_GAME
                    || !game_dirs
                        .iter()
                        .any(|game_dir| game_dir.dir_names().contains(&path_name))
                {
                    copy_paths.push(path);
                }
            }
        }

        info!("Partially copying half-life directory to a second game folder");
        if !no_client_dll_dir.is_dir() {
            fs::create_dir(&no_client_dll_dir).context("Failed to create second game folder")?;
        }

        fs_extra::copy_items(
            copy_paths.as_slice(),
            &no_client_dll_dir,
            &CopyOptions {
                skip_exist: true,
                copy_inside: true,
                ..Default::default()
            },
        )
        .with_context(|| {
            format!(
                "Failed to copy half-life directory from {} to {}",
                hl_dir.display(),
                no_client_dll_dir.display()
            )
        })?;

        // copy the simulator dll to the second half-life directory's steam_api.dll
        info!("Copying simulator client dll to the second half-life directory");
        fs::copy(
            &base_sim_client_dll_path,
            &no_client_dll_dir.join(steam_api_dll),
        )
        .context("Failed to copy simulator dll to the second half-life directory")?;
    }

    // symbolic link cfgs
    cfgs_link(&root_dir, &cfg, minimum_cfgs, reset_cfgs)?;

    // copy default steam_api.dll as _reset.dll
    // only do this on the main half life directory since the no client dll dir is used as the main client
    if reset_dll_path.is_file() {
        info!("_reset.dll already exists, skipping");
    } else {
        info!("Copying default steam_api.dll to _reset.dll");
        fs::copy(&steam_api_dll_path, reset_dll_path)
            .context("Failed to copy steam_api.dll to _reset.dll")?;
    }

    // copy the simulator client steam_api.dll (_sim.dll)
    let sim_client_dll_path = hl_dir.join(sim_dll);

    if sim_client_dll_path.exists() {
        info!("_sim.dll already exists in the Half-Life directory, proceeding copy anyway");
    }

    info!("Copying _sim.dll to the game directory");
    fs::copy(base_sim_client_dll_path, sim_client_dll_path).context("Failed to copy _sim.dll")?;

    stop_tas_script(&root_dir, &cfg)?;

    write_optim_rhai_script(&root_dir, &cfg)?;

    Ok(())
}

fn stop_tas_script<P: AsRef<Path>>(root_dir: P, cfg: &Cfg) -> Result<()> {
    let root_dir = root_dir.as_ref();
    let half_life_dir = root_dir.join(&cfg.half_life_dir);
    let script_name = "stop.hltas";

    info!("Writing stop.hltas script to the Half-Life directory");
    files::write_stop_tas_script(half_life_dir.join(script_name))
        .context("Failed to write stop tas script for Half-Life dir")?;

    if let Some(no_client_dll_dir) = &cfg.no_client_dll_dir {
        let no_client_dll_dir = root_dir.join(no_client_dll_dir);

        info!("Writing stop.hltas script to the second game directory");
        files::write_stop_tas_script(no_client_dll_dir.join(script_name))
            .context("Failed to write stop tas script for second Half-Life dir")?;
    }

    Ok(())
}

fn write_optim_rhai_script<P: AsRef<Path>>(root_dir: P, cfg: &Cfg) -> Result<()> {
    let root_dir = root_dir.as_ref();
    let script_name = "optim.rhai";
    let script_path = root_dir.join(script_name);

    // we write the optim rhai script to the root directory
    if script_path.is_file() {
        info!("optim.rhai already exists, skipping");
    } else {
        info!("Writing optim.rhai script to the root directory");
        files::write_optim_rhai_script(&script_path)?;
    }

    // hard-link to half-life directories
    info!("Hard-linking optim.rhai script to the Half-Life directory");
    force_link(
        &script_path,
        root_dir.join(&cfg.half_life_dir).join(script_name),
    )
    .context("Failed to hard-link optim.rhai script to Half-Life directory")?;

    if let Some(no_client_dll_dir) = &cfg.no_client_dll_dir {
        let no_client_dll_dir = root_dir.join(no_client_dll_dir);

        info!("Hard-linking optim.rhai script to the second game directory");
        force_link(&script_path, no_client_dll_dir.join(script_name))
            .context("Failed to hard-link optim.rhai script to second Half-Life directory")?;
    }

    Ok(())
}

struct ProjectCfgOverrides<'a> {
    projects_dir_name: Option<&'a Path>,
    half_life_dir: Option<&'a Path>,
}

fn cfg_file_set_up<P>(config_path: P, cfg_overrides: ProjectCfgOverrides) -> Result<Cfg>
where
    P: AsRef<Path>,
{
    let config_path = config_path.as_ref();

    if !config_path.is_file() {
        // create the config file
        info!("Creating config file...");
        Cfg::save_default_to_path(&config_path).with_context(|| {
            format!("Failed to create config file at {}", config_path.display())
        })?;
        info!("Created default config file");
    }

    // load config
    let mut cfg = match Cfg::load(&config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            // attempt to save default config to fix the problem
            Cfg::save_default_to_path(&config_path)?;
            let cfg = Cfg::load(&config_path)?;

            warn!("Couldn't load config file, saved default config file");

            cfg
        }
    };

    let mut overridden_cfg = false;

    // save override
    if let Some(projects_dir_name) = cfg_overrides.projects_dir_name {
        cfg.project_dir = projects_dir_name.to_owned();
        overridden_cfg = true;
    }
    if let Some(hl_dir) = cfg_overrides.half_life_dir {
        cfg.half_life_dir = hl_dir.to_owned();
        overridden_cfg = true;
    }

    // save config
    if overridden_cfg {
        cfg.save(&config_path)?;
    }

    Ok(cfg)
}

fn cfgs_link<P>(
    root_dir: P,
    cfg: &Cfg,
    minimum_cfgs: bool,
    reset_cfgs: &Option<Vec<String>>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let root_dir = root_dir.as_ref();
    let half_life_dir = root_dir.join(&cfg.half_life_dir);

    // write cfgs dir
    if let Some(cfgs_dir) = &cfg.cfgs_dir {
        let cfgs_dir = root_dir.join(cfgs_dir);

        info!("Writing tas cfgs in root directory");
        files::write_cfgs(&cfgs_dir, minimum_cfgs, reset_cfgs)?;

        // link to all half-life game directories
        for game_dir in game_dir_types(&half_life_dir)? {
            if !cfg.ignore_games.contains(&game_dir.name) {
                info!(
                    "Linking tas cfgs in main Half-Life dir game {}",
                    game_dir.name
                );
                files::hard_link_cfgs(&cfgs_dir, half_life_dir.join(game_dir.name))?;
            }
        }

        // we link to second client too
        if let Some(no_client_dll_dir) = &cfg.no_client_dll_dir {
            let no_client_dll_dir = root_dir.join(no_client_dll_dir);

            // link to all second half-life game directories
            for game_dir in game_dir_types(&no_client_dll_dir)? {
                if !cfg.ignore_games.contains(&game_dir.name) {
                    info!(
                        "Linking tas cfgs in second Half-Life dir game {}",
                        game_dir.name
                    );
                    files::hard_link_cfgs(&cfgs_dir, no_client_dll_dir.join(game_dir.name))?;
                }
            }
        }
    }

    Ok(())
}
