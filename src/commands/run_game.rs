use std::{
    env::{current_dir, current_exe},
    ffi::OsStr,
    path::Path,
    process::{self, Output},
    thread,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use log::{debug, error, info, warn};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use crate::{
    cfg::{self, Cfg},
    project_toml::{self, ProjectToml},
};

pub struct RunGameMiscFlags {
    pub r_input: bool,
}

pub struct RunGameFlags<'a> {
    pub low: bool,
    pub vanilla_game: bool,
    pub width: u32,
    pub height: u32,
    pub params: &'a Option<Vec<String>>,
    pub game_override: &'a Option<String>,
    pub keep_alive: bool,
}

pub struct RunGameBxtFlags<'a> {
    pub run_script: &'a Option<String>,
    pub optim_games: &'a Option<usize>,
    pub sim: bool,
    pub record: bool,
    pub no_bxt: bool,
}

pub fn run_game(
    run_game_misc_flags: RunGameMiscFlags,
    run_game_flags: RunGameFlags,
    run_game_bxt_flags: RunGameBxtFlags,
) -> Result<()> {
    let RunGameMiscFlags { r_input } = run_game_misc_flags;

    let current_dir_fail = "Failed to get current directory";

    let root_dir = current_exe()
        .expect("failed to get current executable path")
        .parent()
        .expect("failed to get current executable directory")
        .to_path_buf();

    let cfg_dir = root_dir.join(cfg::cfg_file_name());

    info!("Loading config...");
    let cfg = Cfg::load(&cfg_dir)
        .with_context(|| format!("Failed to load config from `{}`", cfg_dir.display()))?;

    let project_dir = {
        let current_dir = current_dir().context(current_dir_fail)?;
        let project_toml = current_dir.join(project_toml::FILE_NAME);

        if project_toml.is_file() {
            Some(current_dir)
        } else {
            None
        }
    };

    info!("Loading project config...");
    let project_toml = match project_dir {
        Some(project_dir) => Some(
            ProjectToml::load_from_path(project_dir.join(project_toml::FILE_NAME))
                .context("Failed to load project config")?,
        ),
        None => None,
    };

    let r_input_exe = root_dir.join("RInput").join("RInput.exe");

    info!("Running game...");
    run_hl(
        root_dir,
        &cfg,
        &project_toml,
        &run_game_flags,
        &run_game_bxt_flags,
    )?;

    if r_input {
        info!("Running RInput...");
        run_r_input(r_input_exe)?;
    }

    Ok(())
}

fn run_r_input<P>(r_input_exe: P) -> Result<Option<Output>>
where
    P: AsRef<Path>,
{
    let r_input_exe = r_input_exe.as_ref();

    if r_input_exe.is_file() {
        let output = process::Command::new(r_input_exe)
            .arg("hl.exe")
            .output()
            .context("Failed to run RInput")?;

        Ok(Some(output))
    } else {
        Ok(None)
    }
}

fn run_hl<P>(
    root_dir: P,
    cfg: &Cfg,
    project_toml: &Option<ProjectToml>,
    run_game_flags: &RunGameFlags,
    run_game_bxt_flags: &RunGameBxtFlags,
) -> Result<Option<Output>>
where
    P: AsRef<Path>,
{
    let RunGameFlags {
        low,
        vanilla_game,
        width,
        height,
        params,
        game_override,
        keep_alive,
    } = run_game_flags;
    let RunGameBxtFlags {
        run_script,
        optim_games,
        sim,
        record,
        no_bxt,
    } = run_game_bxt_flags;

    let root_dir = root_dir.as_ref();
    let injector_exe = root_dir.join("Bunnymod XT").join("Injector.exe");

    let hl_dir = if *vanilla_game || *sim || optim_games.is_some() {
        root_dir.join(&cfg.half_life_dir)
    } else {
        match &cfg.no_client_dll_dir {
            Some(no_client_dll_dir) => root_dir.join(no_client_dll_dir),
            None => bail!("No client DLL dir not set in the config"),
        }
    };
    let hl_exe = hl_dir.join("hl.exe");
    #[cfg(target_os = "linux")]
    let wine_exe = OsStr::new("wine");
    let game = match game_override {
        Some(game_override) => game_override,
        None => match project_toml {
            Some(project_toml) => &project_toml.game,
            None => bail!("No project.toml found\nHelp: Use the game-override parameter"),
        },
    };

    let params = {
        let mut args = Vec::new();

        args.push("-noforcemparms".to_string());
        args.push("-gl".to_string());
        args.push("+gl_vsync 0".to_string());
        args.push("-windowed".to_string());
        args.push(format!("-w {}", width));
        args.push(format!("-h {}", height));

        args.push(format!("-game {game}"));

        if *sim {
            args.push("+bxt_tas_become_simulator_client".to_string());
        }
        if *low {
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
        if *record {
            args.push("-noborder sdl_createwindow".to_string());
        }
        if let Some(run_script) = run_script {
            args.push(format!("+bxt_tas_loadscript {run_script}"));
        }

        if let Some(params) = params {
            for arg in params {
                args.push(arg.to_string());
            }
        }

        debug!("HL args: {:?}", args);

        // intentionally split the args that contains spaces to individual items
        args.iter()
            .flat_map(|arg| arg.split_whitespace().map(|s| s.to_string()))
            .collect::<Vec<_>>()
    };

    let output = if *no_bxt {
        // just run hl.exe
        // no_bxt conflicts with optim_games so no need to run multiple times here
        Some(
            #[cfg(target_os = "linux")]
            process::Command::new(wine_exe)
                .arg(hl_exe)
                .args(params)
                .current_dir(hl_dir)
                .output(),
            #[cfg(target_os = "windows")]
            process::Command::new(hl_exe)
                .args(params)
                .current_dir(hl_dir)
                .output(),
        )
    } else {
        match optim_games {
            Some(optim_games) => {
                let run_game = |out_of, total_games| {
                    #[cfg(target_os = "linux")]
                    let bxt_result = process::Command::new(wine_exe)
                        .arg(&injector_exe)
                        .arg(&hl_exe)
                        .args(&params)
                        .current_dir(&hl_dir)
                        .output();
                    #[cfg(target_os = "windows")]
                    let bxt_result = process::Command::new(&injector_exe)
                        .arg(&hl_exe)
                        .args(&params)
                        .current_dir(&hl_dir)
                        .output();

                    match bxt_result {
                        Ok(_) => info!(
                            "Successfully launched {} out of {} games",
                            out_of, total_games
                        ),
                        // TODO attempt to start the game again if it fails
                        Err(err) => error!(
                            "Failed to launch {} out of {} games: {}",
                            out_of, total_games, err
                        ),
                    }
                };
                let hl_exe_count = |system: &mut System| {
                    // TODO: track pids instead
                    system.refresh_processes_specifics(
                        ProcessesToUpdate::All,
                        true,
                        ProcessRefreshKind::new().with_cpu(),
                    );
                    system.processes_by_exact_name(OsStr::new("hl.exe")).count()
                };

                let mut system = System::new();
                // we get initial half life count
                let initial_hl_count = hl_exe_count(&mut system);

                for i in 0..*optim_games {
                    run_game(i + 1, *optim_games);

                    // TODO wait for the game to start in a better way
                    thread::sleep(Duration::from_secs(6));
                }

                if *keep_alive {
                    loop {
                        // TODO: track pids instead
                        system.refresh_processes_specifics(
                            ProcessesToUpdate::All,
                            true,
                            ProcessRefreshKind::new().with_cpu(),
                        );

                        // scan for hl.exe processes
                        let current_lives = hl_exe_count(&mut system);
                        let expected_lives_count = initial_hl_count + *optim_games;

                        if current_lives < expected_lives_count {
                            let missing_half_lives = expected_lives_count - current_lives;
                            warn!(
                                "Missing {} games, starting up new half-lives",
                                missing_half_lives
                            );

                            for i in 0..missing_half_lives {
                                run_game(i + 1, missing_half_lives);

                                // TODO wait for the game to start in a better way
                                thread::sleep(Duration::from_secs(6));
                            }
                        }

                        thread::sleep(Duration::from_secs(5));
                    }
                }

                None
            }
            None => {
                // run injector with hl.exe as an extra argument
                #[cfg(target_os = "linux")]
                let mut cmd = process::Command::new(wine_exe);
                #[cfg(target_os = "linux")]
                {
                    cmd.arg(injector_exe);
                }
                #[cfg(target_os = "windows")]
                let mut cmd = process::Command::new(injector_exe);

                if let Some(run_script) = run_script {
                    cmd.env("BXT_SCRIPT", run_script);
                }

                cmd.arg(hl_exe).args(params).current_dir(hl_dir);

                Some(cmd.output())
            }
        }
    };

    // TODO INFO: HL output: Some(Ok(Output { status: ExitStatus(ExitStatus(1)), stdout: "", stderr: "E\0r\0r\0o\0r\0...
    // this error can't be picked up, sort it out
    info!("HL output: {:?}", output);

    match output {
        Some(output) => Ok(Some(output.context("Failed to run Half-Life")?)),
        None => Ok(None),
    }
}
