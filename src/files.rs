use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use log::info;
use sha2::Digest;

use crate::{cfg::Cfg, helper};

const HARD_LINK_POST_CHECKOUT_HOOK: &str = include_str!("../resource/git_hooks/post-checkout");
lazy_static! {
    static ref HARD_LINK_POST_CHECKOUT_HOOK_SHA_256: Vec<u8> = {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&HARD_LINK_POST_CHECKOUT_HOOK);
        hasher.finalize().to_vec()
    };
}

pub fn write_hard_link_shell_hook<P>(path: P, cfg: &Cfg) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    let mut file = if path.is_file() {
        // append to existing hook unless it is the same
        if helper::sha_256_file(path).context("Failed to get sha256 of post-checkout hook")?
            == *HARD_LINK_POST_CHECKOUT_HOOK_SHA_256
        {
            info!("Post-checkout hook is already installed");
            return Ok(());
        } else {
            fs::OpenOptions::new()
                .append(true)
                .open(path)
                .context("Failed to open ./git/hooks/post-checkout")?
        }
    } else {
        // create new file
        File::create(path).context("Failed to create ./git/hooks/post-checkout")?
    };

    let hook = {
        let mut hook = HARD_LINK_POST_CHECKOUT_HOOK.replace(
            "HALF_LIFE_DIR",
            &cfg.half_life_dir
                .file_name()
                .context("Failed to get half-life dir name")?
                .to_string_lossy(),
        );

        let no_client_dll_present = "NO_CLIENT_DLL_PRESENT";

        hook = match &cfg.no_client_dll_dir {
            Some(no_client_dll_dir) => hook.replace(no_client_dll_present, "true").replace(
                "NO_CLIENT_DLL_DIR",
                &no_client_dll_dir
                    .file_name()
                    .context("Failed to get no-client-dll dir name")?
                    .to_string_lossy(),
            ),
            None => hook.replace(no_client_dll_present, "false"),
        };

        hook
    };

    info!("Installing post-checkout hook");
    file.write_all(hook.as_bytes())
        .context("Failed to write hard-link hook to ./git/hooks/post-checkout")?;

    Ok(())
}

const HLTAS_CFG: &[u8] = include_bytes!("../resource/cfgs/hltas.cfg");
const INGAME_CFG: &[u8] = include_bytes!("../resource/cfgs/ingame.cfg");
const RECORD_CFG: &[u8] = include_bytes!("../resource/cfgs/record.cfg");
const EDITOR_CFG: &[u8] = include_bytes!("../resource/cfgs/editor.cfg");
const CAM_CFG: &[u8] = include_bytes!("../resource/cfgs/cam.cfg");

const HLTAS_MIN_CFG: &[u8] = include_bytes!("../resource/cfgs/hltas_min.cfg");
const INGAME_MIN_CFG: &[u8] = include_bytes!("../resource/cfgs/ingame_min.cfg");
const RECORD_MIN_CFG: &[u8] = include_bytes!("../resource/cfgs/record_min.cfg");
const EDITOR_MIN_CFG: &[u8] = include_bytes!("../resource/cfgs/editor_min.cfg");
const CAM_MIN_CFG: &[u8] = include_bytes!("../resource/cfgs/cam_min.cfg");

pub fn write_cfgs<P>(path: P, minimum: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.is_dir() {
        fs::create_dir_all(path).context("Failed to create cfg dir")?;
    }

    let files = if minimum {
        vec![
            ("hltas.cfg", HLTAS_MIN_CFG),
            ("ingame.cfg", INGAME_MIN_CFG),
            ("record.cfg", RECORD_MIN_CFG),
            ("editor.cfg", EDITOR_MIN_CFG),
            ("cam.cfg", CAM_MIN_CFG),
        ]
    } else {
        vec![
            ("hltas.cfg", HLTAS_CFG),
            ("ingame.cfg", INGAME_CFG),
            ("record.cfg", RECORD_CFG),
            ("editor.cfg", EDITOR_CFG),
            ("cam.cfg", CAM_CFG),
        ]
    };

    for (file_name, cfg_file) in files {
        let path = path.join(&file_name);

        if path.is_file() {
            info!("Config {file_name} already exists, skipping");
        } else {
            let mut file = File::create(&path)?;
            file.write_all(cfg_file).with_context(|| {
                format!(
                    "Failed to write cfg file {} to {}",
                    file_name,
                    path.display()
                )
            })?;
        }
    }

    Ok(())
}

pub fn hard_link_cfgs<P, P2>(cfgs_dir: P, dest_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let cfgs_dir = cfgs_dir.as_ref();
    let dest_dir = dest_dir.as_ref();

    let files = vec![
        "hltas.cfg",
        "ingame.cfg",
        "record.cfg",
        "editor.cfg",
        "cam.cfg",
    ];

    for file_name in files {
        let src_path = cfgs_dir.join(file_name);
        let dest_path = dest_dir.join(file_name);

        if !src_path.exists() {
            bail!("cfg in {} does not exist", &src_path.display());
        }

        // delete it if the file already exists
        if dest_path.exists() {
            info!("Config {file_name} is already hard-linked, deleting");
            fs::remove_file(&dest_path).context("Failed to delete hard-linked cfg")?;
        }

        fs::hard_link(&src_path, &dest_path).with_context(|| {
            format!(
                "Failed to hard-link {} to {}",
                &src_path.display(),
                &dest_path.display()
            )
        })?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
const ENABLE_VANILLA_GAME: &str = include_str!("../resource/bat/enable_vanilla_game.bat");
#[cfg(target_os = "windows")]
const DISABLE_VANILLA_GAME: &str = include_str!("../resource/bat/disable_vanilla_game.bat");

#[cfg(target_os = "windows")]
pub fn write_toggle_vanilla_game<P, P2>(path: P, game_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.is_dir() {
        bail!("{} is not a directory", path.display());
    }

    let client_dll_path = game_dir.as_ref().join("cl_dlls");
    let client_dll_path_str = client_dll_path.to_string_lossy();

    let enable_vanilla_game = ENABLE_VANILLA_GAME.replace("GAME_DIR", &client_dll_path_str);
    let disable_vanilla_game = DISABLE_VANILLA_GAME.replace("GAME_DIR", &client_dll_path_str);

    let files = vec![
        ("disable_vanilla_game", disable_vanilla_game),
        ("enable_vanilla_game", enable_vanilla_game),
    ];

    for (file_name, cfg_file) in files {
        let file_name = format!("{}.bat", file_name);

        let mut file = File::create(path.join(&file_name))?;
        file.write_all(cfg_file.as_bytes())
            .with_context(|| format!("Failed to write file {} to {}", file_name, path.display()))?;
    }

    Ok(())
}

#[cfg(all(not(target_os = "windows")))]
pub fn write_toggle_vanilla_game<P, P2>(path: P, game_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    compile_error!("Toggle vanilla game is not implemented for this platform");
}

#[cfg(target_os = "windows")]
const ENABLE_SIM_CLIENT: &str = include_str!("../resource/bat/enable_sim_client.bat");
#[cfg(target_os = "windows")]
const DISABLE_SIM_CLIENT: &str = include_str!("../resource/bat/disable_sim_client.bat");

#[cfg(target_os = "windows")]
pub fn write_toggle_sim_client<P, P2>(dir: P, half_life_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let dir = dir.as_ref();

    if !dir.is_dir() {
        bail!("{} is not a directory", dir.display());
    }

    let half_life_dir = half_life_dir
        .as_ref()
        .file_name()
        .context("Failed to get half-life dir")?;

    let enable_sim_client =
        ENABLE_SIM_CLIENT.replace("HALF_LIFE_DIR", &half_life_dir.to_string_lossy());

    let disable_sim_client =
        DISABLE_SIM_CLIENT.replace("HALF_LIFE_DIR", &half_life_dir.to_string_lossy());

    let files = vec![
        ("disable_sim_client.bat", disable_sim_client),
        ("enable_sim_client.bat", enable_sim_client),
    ];

    for (file_name, cfg_file) in files {
        let mut file = File::create(dir.join(&file_name))?;
        file.write_all(cfg_file.as_bytes())
            .with_context(|| format!("Failed to write file {} to {}", file_name, dir.display()))?;
    }

    Ok(())
}

#[cfg(all(not(target_os = "windows")))]
pub fn write_toggle_sim_client<P, P2>(_dir: P, _half_life_dir: P2) -> Result<()>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    compile_error!("write_toggle_sim_client is not implemented for this platform");
}

#[cfg(target_os = "windows")]
const RUN_MANAGER_EXEC_BASE_BAT: &str = include_str!("../resource/bat/run_manager_base.bat");
#[cfg(target_os = "windows")]
const RUN_MANAGER_EXEC_BASE_PS1: &str = include_str!("../resource/ps1/run_manager_base.ps1");
#[cfg(target_os = "windows")]
const RUN_MANAGER_BAT: &str = include_str!("../resource/bat/run_manager.bat");
#[cfg(target_os = "windows")]
const RUN_MANAGER_PS1: &str = include_str!("../resource/ps1/run_manager.ps1");

const RUN_MANAGER_SUB_COMMAND: &str = "SUB_COMMAND";

#[cfg(target_os = "windows")]
pub fn write_manager_script_bat<P>(dest: P, sub_command: Option<&str>) -> Result<()>
where
    P: AsRef<Path>,
{
    let dest = dest.as_ref();

    let script = match sub_command {
        Some(sub_command) => {
            RUN_MANAGER_EXEC_BASE_BAT.replace(RUN_MANAGER_SUB_COMMAND, sub_command)
        }
        None => RUN_MANAGER_BAT.to_string(),
    };

    let mut file = File::create(dest)
        .with_context(|| format!("Failed to create file at {}", dest.display()))?;
    file.write_all(script.as_bytes())
        .with_context(|| format!("Failed to write file to {}", dest.display()))?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn write_manager_script<P>(dest: P, sub_command: Option<&str>) -> Result<()>
where
    P: AsRef<Path>,
{
    let dest = dest.as_ref();

    let script = match sub_command {
        Some(sub_command) => {
            RUN_MANAGER_EXEC_BASE_PS1.replace(RUN_MANAGER_SUB_COMMAND, sub_command)
        }
        None => RUN_MANAGER_PS1.to_string(),
    };

    let mut file = File::create(dest)
        .with_context(|| format!("Failed to create file at {}", dest.display()))?;
    file.write_all(script.as_bytes())
        .with_context(|| format!("Failed to write file to {}", dest.display()))?;

    Ok(())
}

#[cfg(all(not(target_os = "windows")))]
pub fn write_manager_script<P>(_path: P, _file_name: &str, _sub_command: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    compile_error!("write_run_manager_sub_command_script is not implemented for this platform");
}

pub fn write_stop_tas_script<P: AsRef<Path>>(path: P) -> Result<()> {
    let script = include_str!("../resource/hltas/stop.hltas");

    fs::write(path, script).context("Failed to write stop.hltas")
}

pub fn write_optim_rhai_script<P: AsRef<Path>>(path: P) -> Result<()> {
    let script = include_str!("../resource/rhai/optim.rhai");

    fs::write(path, script).context("Failed to write optim.rhai")
}
