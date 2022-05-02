use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use sha2::Digest;

use crate::cfg;

pub fn root_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().context("Failed to get current exe path")?;
    Ok(exe_path
        .parent()
        .context("Failed to get exe dir")?
        .to_path_buf())
}

pub fn cfg_dir() -> Result<PathBuf> {
    let root = root_dir()?;
    let file_name = cfg::cfg_file_name();

    Ok(root.join(file_name))
}

pub fn sha_256_file<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    let file = fs::read(&path)
        .with_context(|| format!("Failed to open file {}", path.as_ref().display()))?;
    let mut hasher = sha2::Sha256::new();

    hasher.update(&file);

    Ok(hasher.finalize().to_vec())
}

#[cfg(target_os = "windows")]
pub fn move_window_to_pos(x: i32, y: i32, process_name: &str) -> Result<()> {
    use winapi::um::winuser::{SetWindowPos, HWND_NOTOPMOST, SWP_NOSIZE, SWP_SHOWWINDOW};

    let hwnd = unsafe {
        winapi::um::winuser::FindWindowA(
            std::ptr::null(),
            process_name.as_bytes().as_ptr() as *const i8,
        )
    };

    if hwnd == std::ptr::null_mut() {
        bail!("Failed to find window");
    }

    unsafe {
        SetWindowPos(
            hwnd,
            HWND_NOTOPMOST,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_SHOWWINDOW,
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn move_window_to_pos(x: i32, y: i32, process_name: &str) -> Result<()> {
    let mut cmd = process::Command::new("wmctrl");
    cmd.arg("-r")
        .arg(process_name)
        .arg("-e")
        .arg(format!("0,{},{},0,0", x, y));

    let output = cmd.output().context("Failed to move window")?;

    if !output.status.success() {
        bail!("wmctrl failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "linux")))]
pub fn move_window_to_pos(_x: i32, _y: i32, _process_name: &str) -> Result<()> {
    compile_error!("move_window_to_pos is not implemented for this platform");
}
