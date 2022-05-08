use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};

use anyhow::{bail, Context, Result};
#[cfg(target_os = "windows")]
use lazy_static::lazy_static;
use sha2::Digest;
#[cfg(target_os = "windows")]
use std::sync::Mutex;
use sysinfo::{ProcessRefreshKind, RefreshKind, System, SystemExt};
#[cfg(target_os = "windows")]
use winapi::{
    shared::minwindef::FALSE,
    um::winuser::{SetWindowPos, HWND_NOTOPMOST, SWP_NOSIZE, SWP_SHOWWINDOW},
};

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
pub fn move_window_to_pos(x: i32, y: i32, window_title: &str) -> Result<()> {
    use winapi::{
        shared::{
            minwindef::{BOOL, LPARAM, TRUE},
            windef::HWND,
        },
        um::winuser::{EnumWindows, GetWindowTextA, GetWindowTextLengthA},
    };

    // hack to pass arguments to the callback function
    lazy_static! {
        static ref WINDOW_TITLE: Mutex<String> = Mutex::new(String::new());
        static ref X: Mutex<i32> = Mutex::new(0);
        static ref Y: Mutex<i32> = Mutex::new(0);
    }

    *WINDOW_TITLE.lock().unwrap() = window_title.to_string();
    *X.lock().unwrap() = x;
    *Y.lock().unwrap() = y;

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut title = Vec::with_capacity(1024);

        GetWindowTextA(hwnd, title.as_mut_ptr(), title.capacity() as i32);
        title.set_len(GetWindowTextLengthA(hwnd) as usize);

        let title = title.into_iter().map(|c| c as u8).collect::<Vec<u8>>();
        if let Ok(title) = String::from_utf8(title) {
            if title == *WINDOW_TITLE.lock().unwrap() {
                SetWindowPos(
                    hwnd,
                    HWND_NOTOPMOST,
                    *X.lock().unwrap(),
                    *Y.lock().unwrap(),
                    0,
                    0,
                    SWP_NOSIZE | SWP_SHOWWINDOW,
                );
            }
        }

        TRUE
    }

    let enum_success = unsafe { EnumWindows(Some(enum_windows_proc), 0) };

    if enum_success == FALSE {
        bail!("Failed to enumerate windows");
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
