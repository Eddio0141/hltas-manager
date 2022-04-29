use std::{
    path::Path,
    process::{self, Output},
};

use anyhow::{Context, Result};
use log::info;

use crate::{
    cfg::Cfg,
    helper::{cfg_dir, root_dir},
};

pub struct RunGameFlags {
    pub sim: bool,
    pub low: bool,
    pub no_vanilla: bool,
    pub record: bool,
    pub no_bxt: bool,
    pub no_r_input: bool,
}

pub fn run_game(
    run_game_flags: RunGameFlags,
    width: u32,
    height: u32,
    run_script: &Option<String>,
    params: &Option<Vec<String>>,
) -> Result<()> {
    let root_dir = root_dir()?;

    info!("Loading config...");
    let cfg_dir = cfg_dir()?;
    let cfg = Cfg::load_from_path(cfg_dir).context("Failed to load cfg")?;

    let half_life_dir = root_dir.join(&cfg.half_life_dir);
    let bxt_injector_exe = root_dir.join("Bunnymod XT").join("Injector.exe");
    let r_input_exe = root_dir.join("RInput").join("RInput.exe");
    let tas_view_dir = root_dir.join("TASView");

    info!("Running game...");

    todo!()
}

fn run_r_input<P>(r_input_exe: P) -> Result<Option<Output>>
where
    P: AsRef<Path>,
{
    let r_input_exe = r_input_exe.as_ref();

    if r_input_exe.is_file() {
        info!("Running RInput...");
        let output = process::Command::new(r_input_exe)
            .output()
            .context("Failed to run RInput")?;

        Ok(Some(output))
    } else {
        Ok(None)
    }
}

fn run_tas_view<P>(tas_view_exe: P) -> Result<Option<Output>>
where
    P: AsRef<Path>,
{
    let tas_view_exe = tas_view_exe.as_ref();

    if tas_view_exe.is_file() {
        info!("Running TASView...");
        let output = process::Command::new(tas_view_exe)
            .output()
            .context("Failed to run TASView")?;

        // TODO place TASView to the left of the screen

        Ok(Some(output))
    } else {
        Ok(None)
    }
}

fn run_hl<P, P2>(
    hl_dir: P,
    hl_exe_flags: Vec<String>,
    injector_exe: P2,
    run_game_flags: RunGameFlags,
) -> Result<Output>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    // TODO depending if we are running a simulator client, we use the secondary game dir
    let hl_dir = hl_dir.as_ref();
    let hl_exe = hl_dir.join("hl.exe");

    let flag_args = {
        let mut flags = Vec::new();

        if run_game_flags.sim {
            flags.push("+bxt_tas_become_simulator_client");
        }
        if run_game_flags.low {
            //-nofbo -nomsaa +gl_spriteblend 0 +r_detailtextures 0 -gl_ansio 0 +gl_texturemode GL_Nearest +gl_round_down 0 +violence_ablood 0 +violence_agibs 0 +violence_hblood 0 +violence_hgibs 0
            flags.push("-nofbo");
            flags.push("-nomsaa");
            flags.push("+gl_spriteblend 0");
            flags.push("+r_detailtextures 0");
            flags.push("-gl_ansio 0");
            flags.push("+gl_texturemode GL_Nearest");
        }
        if run_game_flags.record {
            flags.push("-noborder sdl_createwindow");
        }

        flags
    };

    let output = if run_game_flags.no_bxt {
        // just run hl.exe
        let output = process::Command::new(hl_exe)
            .args(hl_exe_flags)
            .current_dir(hl_dir)
            .output()
            .context("Failed to run Half-Life")?;

        output
    } else {
        todo!()
    };

    Ok(output)
}
