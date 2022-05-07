use std::{
    fs::{self},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};

use anyhow::{bail, Context, Result};
use sha2::Digest;
use sysinfo::{ProcessRefreshKind, RefreshKind, System, SystemExt};

use crate::cfg;

pub struct CurrentDir {
    pub path: PathBuf,
    pub location: DirLocation,
}

pub enum DirLocation {
    Project,
    Root,
}

pub fn try_root_dir() -> Result<CurrentDir> {
    let working_dir = std::env::current_dir().context("Failed to get current dir")?;

    let root_cfg_path = working_dir.join(cfg::cfg_file_name());

    if root_cfg_path.is_file() {
        Ok(CurrentDir {
            path: working_dir,
            location: DirLocation::Root,
        })
    } else {
        let tas_dir = working_dir.parent().context("Failed to get projects dir")?;
        let root_dir = tas_dir.parent().context("Failed to get root dir")?;
        let root_cfg_path = root_dir.join(cfg::cfg_file_name());

        if root_cfg_path.is_file() {
            Ok(CurrentDir {
                path: root_dir.to_path_buf(),
                location: DirLocation::Project,
            })
        } else {
            bail!("Using the program from an unknown location");
        }
    }
}

pub fn exe_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().context("Failed to get current exe path")?;
    Ok(exe_path
        .parent()
        .context("Failed to get exe dir")?
        .to_path_buf())
}

pub fn cfg_dir() -> Result<PathBuf> {
    let root = exe_dir()?;
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
    use std::mem::size_of;

    use winapi::{
        shared::minwindef::{FALSE, MAX_PATH},
        um::{
            handleapi::CloseHandle,
            tlhelp32::{
                CreateToolhelp32Snapshot, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
            },
            winuser::{SetWindowPos, HWND_NOTOPMOST, SWP_NOSIZE, SWP_SHOWWINDOW},
        },
    };

    let process_name = process_name.bytes().map(|b| b as i8).collect::<Vec<i8>>();

    let hwnd = {
        let mut process_entry = PROCESSENTRY32 {
            dwSize: size_of::<PROCESSENTRY32>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; MAX_PATH],
        };

        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

        if snapshot.is_null() {
            bail!("Failed to create snapshot");
        }

        let mut first = true;
        let mut hwnd = None;

        loop {
            if first {
                first = false;
            } else {
                let has_next = unsafe { Process32Next(snapshot, &mut process_entry) };

                if has_next == FALSE {
                    break;
                }
            }

            if process_entry.szExeFile == process_name.as_slice() {
                hwnd = Some(process_entry.th32ProcessID);
                break;
            }
        }

        unsafe {
            CloseHandle(snapshot);
        }

        hwnd
    };

    let hwnd = match hwnd {
        Some(hwnd) => hwnd as *mut _,
        None => bail!("Failed to find window"),
    };

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

    // close handle
    unsafe {
        CloseHandle(hwnd as *mut _);
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

pub fn wait_for_process_exit(name: &str, timeout: Duration) -> Result<()> {
    let start_time = SystemTime::now();
    let end_time = start_time + timeout;
    let mut sys = System::new_with_specifics(
        RefreshKind::default()
            .with_processes(ProcessRefreshKind::everything().without_disk_usage()),
    );

    loop {
        if sys.processes_by_exact_name(name).next().is_none() {
            return Ok(());
        }

        let now = SystemTime::now();
        if now > end_time {
            bail!("Process {} did not exit in time", name);
        }

        thread::sleep(std::time::Duration::from_millis(100));

        sys.refresh_processes();
    }
}

pub fn wait_for_process_start(name: &str, timeout: Duration) -> Result<()> {
    let start_time = SystemTime::now();
    let end_time = start_time + timeout;
    let mut sys = System::new_with_specifics(
        RefreshKind::default()
            .with_processes(ProcessRefreshKind::everything().without_disk_usage()),
    );

    loop {
        if sys.processes_by_exact_name(name).next().is_some() {
            return Ok(());
        }

        let now = SystemTime::now();
        if now > end_time {
            bail!("Process {} did not start in time", name);
        }

        thread::sleep(std::time::Duration::from_millis(100));

        sys.refresh_processes();
    }
}
