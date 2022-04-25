use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{bail, Context, Result};

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

// TODO review those cfg files
pub const HLTAS_CFG: &[u8] = include_bytes!("./files/cfgs/hltas.cfg");
pub const INGAME_CFG: &[u8] = include_bytes!("./files/cfgs/ingame.cfg");
pub const RECORD_CFG: &[u8] = include_bytes!("./files/cfgs/record.cfg");
pub const EDITOR_CFG: &[u8] = include_bytes!("./files/cfgs/editor.cfg");
pub const CAM_CFG: &[u8] = include_bytes!("./files/cfgs/cam.cfg");

pub fn write_cfgs<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.is_dir() {
        bail!("{} is not a directory", path.display());
    }

    let files = vec![("hltas", HLTAS_CFG)];

    for (file_name, cfg_file) in files {
        let file_name = format!("{}.cfg", file_name);

        let mut file = File::create(path.join(&file_name))?;
        file.write_all(cfg_file).with_context(|| {
            format!(
                "Failed to write cfg file {} to {}",
                file_name,
                path.display()
            )
        })?;
    }

    Ok(())
}

pub const ENABLE_VANILLA_GAME: &str = include_str!("./files/bat/enable_vanilla_game.bat");
pub const DISABLE_VANILLA_GAME: &str = include_str!("./files/bat/disable_vanilla_game.bat");

pub fn write_toggle_vanilla_game<P>(path: P, game_dir: P) -> Result<()>
where
    P: AsRef<Path>,
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

pub const ENABLE_SIM_CLIENT: &str = include_str!("./files/bat/enable_sim_client.bat");
pub const DISABLE_SIM_CLIENT: &str = include_str!("./files/bat/disable_sim_client.bat");

pub fn write_toggle_sim_client<P>(dir: P, half_life_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let dir = dir.as_ref();

    if !dir.is_dir() {
        bail!("{} is not a directory", dir.display());
    }

    let enable_sim_client =
        ENABLE_SIM_CLIENT.replace("HALF_LIFE_DIR", &half_life_dir.as_ref().to_string_lossy());
    let disable_sim_client =
        DISABLE_SIM_CLIENT.replace("HALF_LIFE_DIR", &half_life_dir.as_ref().to_string_lossy());

    let files = vec![
        ("disable_sim_client.bat", disable_sim_client),
        ("enable_sim_client.bat", enable_sim_client),
    ];

    for (file_name, cfg_file) in files {
        let file_name = format!("{}.bat", file_name);

        let mut file = File::create(dir.join(&file_name))?;
        file.write_all(cfg_file.as_bytes())
            .with_context(|| format!("Failed to write file {} to {}", file_name, dir.display()))?;
    }

    Ok(())
}

pub const LINK_HLTAS_FILES: &str = include_str!("./files/bat/link_hltas_files.bat");

pub fn write_hltas_linker<P>(dir: P, half_life_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let linker_file = LINK_HLTAS_FILES.replace(
        "HALF_LIFE_DIR",
        half_life_dir.as_ref().to_string_lossy().as_ref(),
    );

    let mut file = File::create(dir.as_ref().join("link_hltas_files.bat"))?;

    file.write_all(linker_file.as_bytes()).with_context(|| {
        format!(
            "Failed to write file link_hltas_files.bat to {}",
            dir.as_ref().display()
        )
    })?;

    Ok(())
}
