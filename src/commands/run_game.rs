use std::{
    path::Path,
    process::{self, Output},
};

use anyhow::{bail, Context, Result};
use log::info;

use crate::{
    cfg::Cfg,
    helper::{cfg_dir, root_dir},
};

pub struct RunGameFlags {
    pub sim: bool,
    pub low: bool,
    pub vanilla_game: bool,
    pub record: bool,
    pub no_bxt: bool,
    pub r_input: bool,
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
            .arg("hl.exe")
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
    hl_exe_args: Vec<String>,
    injector_exe: P2,
    run_game_flags: RunGameFlags,
    cfg: &Cfg,
    width: u32,
    height: u32,
    run_script: &Option<String>,
) -> Result<Output>
where
    P: AsRef<Path>,
    P2: AsRef<Path>,
{
    let hl_dir = if run_game_flags.vanilla_game || run_game_flags.sim {
        hl_dir.as_ref()
    } else {
        match &cfg.no_client_dll_dir {
            Some(no_client_dll_dir) => no_client_dll_dir.as_path(),
            None => bail!("No client DLL dir not set in the config"),
        }
    };
    let hl_exe = hl_dir.join("hl.exe");
    let injector_exe = injector_exe.as_ref();
    // TODO use of project.toml
    let game = "valve";

    let hl_exe_args = {
        let mut args = Vec::new();

        args.push("-noforcemparms".to_string());
        args.push("-gl".to_string());
        args.push("+gl_vsync 0".to_string());
        args.push("+exec userconfig.cfg".to_string());
        args.push(format!("-w {}", width));
        args.push(format!("-h {}", height));

        args.push(format!("-game {game}"));

        if run_game_flags.sim {
            args.push("+bxt_tas_become_simulator_client".to_string());
        }
        if run_game_flags.low {
            args.push("-nofbo".to_string());
            args.push("-nomsaa".to_string());
            args.push("+gl_spriteblend 0".to_string());
            args.push("+r_detailtextures 0".to_string());
            args.push("-gl_ansio 0".to_string());
            args.push("+gl_texturemode GL_Nearest".to_string());
            args.push("+gl_round_down 0".to_string());
            args.push("+violence_ablood 0".to_string());
            args.push("+violence_agibs 0".to_string());
            args.push("+violence_hblood 0".to_string());
            args.push("+violence_hgibs 0".to_string());
        }
        if !run_game_flags.vanilla_game {
            // TODO
        }
        if run_game_flags.record {
            args.push("-noborder sdl_createwindow".to_string());
        }
        if let Some(run_script) = run_script {
            args.push(format!("+bxt_tas_loadscript {run_script}"));
        }

        for arg in hl_exe_args {
            args.push(arg);
        }

        args
    };

    let output = if run_game_flags.no_bxt {
        // just run hl.exe
        process::Command::new(hl_exe)
            .args(hl_exe_args)
            .current_dir(hl_dir)
            .output()
    } else {
        // run injector with hl.exe as an extra argument
        let mut cmd = process::Command::new(injector_exe);

        if let Some(run_script) = run_script {
            cmd.env("BXT_SCRIPT", run_script);
        }

        cmd.arg(hl_exe).args(hl_exe_args).current_dir(hl_dir);

        cmd.output()
    }
    .context("Failed to run Half-Life")?;

    Ok(output)
}
