use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

use anyhow::{bail, Context, Result};
use fs_extra::dir::CopyOptions;

use crate::{
    cfg::Cfg,
    files,
    helper::{self, root_dir},
    DEFAULT_GAME,
};

pub const STEAM_API_DLL_HASH: &[u8] = &[
    0x8c, 0x07, 0x3e, 0x0d, 0x2c, 0xa3, 0x9d, 0x1e, 0x98, 0x6b, 0xec, 0x34, 0x8f, 0x98, 0x83, 0x03,
    0x35, 0x7b, 0xe5, 0xc4, 0x95, 0xcc, 0xf6, 0xe0, 0x41, 0x58, 0x02, 0xb8, 0x6e, 0xae, 0x35, 0x34,
];

pub fn install(
    projects_dir: &Option<PathBuf>,
    half_life_dir: &Option<PathBuf>,
    minimum_cfgs: bool,
) -> Result<()> {
    // config
    let config_path = helper::cfg_dir()?;
    let cfg = cfg_file_set_up(
        config_path,
        CfgOverrides {
            projects_dir_name: projects_dir.as_deref(),
            half_life_dir: half_life_dir.as_deref(),
        },
    )?;

    // paths
    let root_dir = root_dir()?;
    let hl_dir = root_dir.join(&cfg.half_life_dir);
    let projects_dir = root_dir.join(&cfg.project_dir);
    let sim_dll = "sim.dll";
    let base_sim_client_dll_path = root_dir.join(sim_dll);
    let steam_api_dll = "steam_api.dll";
    let steam_api_dll_path = hl_dir.join(steam_api_dll);
    let reset_dll_path = hl_dir.join("reset.dll");

    // verifying if the half-life directory exists
    if !hl_dir.is_dir() {
        bail!("Half-life directory does not exist");
    }
    // verifying that sim.dll exists in root dir
    if !base_sim_client_dll_path.is_file() {
        bail!("sim.dll does not exist in the root directory");
    }

    // verify if steam_api.dll hash is matching
    let steam_api_dll_hash = helper::sha_256_file(&steam_api_dll_path)?;

    if STEAM_API_DLL_HASH != steam_api_dll_hash.as_slice() {
        // check if reset.dll exists and that hash is matching
        // TODO skip steam api stuff if not matching and warn user
        // TODO but if the files reset.dll and sim.dll exists with the same hash, then we can skip this
    }

    // hard link cfgs
    cfgs_link(&root_dir, &cfg, minimum_cfgs)?;

    // create projects dir if it doesn't exist
    let projects_dir_create_worker = if !projects_dir.is_dir() {
        if cfg.project_dir.parent().is_some() {
            bail!("Projects directory needs to be inside the root dir without any directories in between");
        }

        Some(thread::spawn(move || {
            fs::create_dir(&projects_dir).context("Failed to create projects directory")
        }))
    } else {
        None
    };

    // copy half life directory if needs to be copied
    if let Some(no_client_dll_dir) = &cfg.no_client_dll_dir {
        let no_client_dll_dir = root_dir.join(no_client_dll_dir);

        if !no_client_dll_dir.is_dir() {
            // TODO don't copy games in ignore list and exclude from copying here
            fs::create_dir(&no_client_dll_dir).context("Failed to create second game folder")?;
            fs_extra::dir::copy(
                &hl_dir,
                &no_client_dll_dir,
                &CopyOptions {
                    overwrite: true,
                    content_only: true,
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
            fs::copy(
                &base_sim_client_dll_path,
                &no_client_dll_dir.join(steam_api_dll),
            )
            .context("Failed to copy simulator dll to the second half-life directory")?;
        }
    }

    // copy default steam_api.dll as reset.dll
    // only on main half life directory
    let sim_dll_copy_worker = if steam_api_dll_path.is_file() {
        thread::spawn(move || {
            fs::copy(&steam_api_dll_path, reset_dll_path)
                .context("Failed to copy steam_api.dll to reset.dll")
        })
    } else {
        bail!("steam_api.dll not found in the Half-Life directory");
    };

    // copy the simulator client steam_api.dll (sim.dll)
    // only do this on the main half life directory since the no client dll dir is used as the main client
    let steam_dll_copy_worker = if base_sim_client_dll_path.is_file() {
        let sim_client_dll_path = hl_dir.join(sim_dll);

        thread::spawn(move || {
            fs::copy(base_sim_client_dll_path, sim_client_dll_path)
                .context("Failed to copy sim.dll")
        })
    } else {
        bail!(format!("{sim_dll} not found in the root directory"));
    };

    if let Some(projects_dir_create_worker) = projects_dir_create_worker {
        match projects_dir_create_worker.join() {
            Ok(res) => res?,
            Err(_) => bail!("Failed to create projects directory"),
        }
    }
    match sim_dll_copy_worker.join() {
        Ok(res) => res?,
        Err(_) => bail!("Failed to copy sim.dll"),
    };
    match steam_dll_copy_worker.join() {
        Ok(res) => res?,
        Err(_) => bail!("Failed to copy steam_api.dll"),
    };

    Ok(())
}

struct CfgOverrides<'a> {
    projects_dir_name: Option<&'a Path>,
    half_life_dir: Option<&'a Path>,
}

fn cfg_file_set_up<P>(config_path: P, cfg_overrides: CfgOverrides) -> Result<Cfg>
where
    P: AsRef<Path>,
{
    let config_path = config_path.as_ref();

    if !config_path.is_file() {
        // create the config file
        Cfg::save_default_to_path(&config_path)?;
    }

    // load config
    let mut cfg = match Cfg::load_from_path(&config_path) {
        Ok(cfg) => cfg,
        Err(_) => {
            // attempt to save default config to fix the problem
            Cfg::save_default_to_path(&config_path)?;
            Cfg::load_from_path(&config_path)?
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
        cfg.save_to_path(&config_path)?;
    }

    Ok(cfg)
}

fn cfgs_link<P>(root_dir: P, cfg: &Cfg, minimum_cfgs: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let root_dir = root_dir.as_ref();
    let half_life_dir = root_dir.join(&cfg.half_life_dir);

    // write cfgs dir
    if let Some(cfgs_dir) = &cfg.cfgs_dir {
        let cfgs_dir = root_dir.join(cfgs_dir);

        files::write_cfgs(&cfgs_dir, minimum_cfgs)?;

        // link to default game
        let default_game_dir = half_life_dir.join(DEFAULT_GAME);

        files::hard_link_cfgs(&cfgs_dir, default_game_dir)?;
    }

    Ok(())
}
