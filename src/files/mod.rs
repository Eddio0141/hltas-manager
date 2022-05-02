use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{bail, Context, Result};
use log::info;

pub const HARD_LINK_POST_CHECKOUT_HOOK: &[u8] = include_bytes!("./files/git_hooks/post-checkout");

pub fn write_hard_link_shell_hook<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    let mut file = if path.is_file() {
        // append to existing hook

        fs::OpenOptions::new()
            .append(true)
            .open(path)
            .context("Failed to open ./git/hooks/post-checkout")?
    } else {
        // create new file

        File::create(path).context("Failed to create ./git/hooks/post-checkout")?
    };

    file.write_all(HARD_LINK_POST_CHECKOUT_HOOK)
        .context("Failed to write hard-link hook to ./git/hooks/post-checkout")?;

    Ok(())
}

pub const HLTAS_CFG: &[u8] = include_bytes!("./files/cfgs/hltas.cfg");
pub const INGAME_CFG: &[u8] = include_bytes!("./files/cfgs/ingame.cfg");
pub const RECORD_CFG: &[u8] = include_bytes!("./files/cfgs/record.cfg");
pub const EDITOR_CFG: &[u8] = include_bytes!("./files/cfgs/editor.cfg");
pub const CAM_CFG: &[u8] = include_bytes!("./files/cfgs/cam.cfg");

pub const HLTAS_MIN_CFG: &[u8] = include_bytes!("./files/cfgs/hltas_min.cfg");
pub const INGAME_MIN_CFG: &[u8] = include_bytes!("./files/cfgs/ingame_min.cfg");
pub const RECORD_MIN_CFG: &[u8] = include_bytes!("./files/cfgs/record_min.cfg");
pub const EDITOR_MIN_CFG: &[u8] = include_bytes!("./files/cfgs/editor_min.cfg");
pub const CAM_MIN_CFG: &[u8] = include_bytes!("./files/cfgs/cam_min.cfg");

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

        // ignore operation if the file already exists
        if dest_path.exists() {
            info!("Config {file_name} is already hard-linked, skipping");
        } else {
            fs::hard_link(&src_path, &dest_path).with_context(|| {
                format!(
                    "Failed to hard-link {} to {}",
                    &src_path.display(),
                    &dest_path.display()
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub const ENABLE_VANILLA_GAME: &str = include_str!("./files/bat/enable_vanilla_game.bat");
#[cfg(target_os = "windows")]
pub const DISABLE_VANILLA_GAME: &str = include_str!("./files/bat/disable_vanilla_game.bat");

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
pub const ENABLE_SIM_CLIENT: &str = include_str!("./files/bat/enable_sim_client.bat");
#[cfg(target_os = "windows")]
pub const DISABLE_SIM_CLIENT: &str = include_str!("./files/bat/disable_sim_client.bat");

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
