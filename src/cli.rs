use std::path::PathBuf;

use clap::*;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    // TODO actually implement those flags
    #[clap(long)]
    pub quiet: bool,
    #[clap(long)]
    pub no_colour: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Sets up the required files for the tool.
    ///
    /// - Can be used again to verify if the files are already installed.
    /// - Requires the 'steamapi.dll' file to be the default dll.
    /// - Needs to be run before the tool can be used.
    ///
    /// - Creates the cfg files 'hltas.cfg', 'ingame.cfg', 'record.cfg', 'editor.cfg' and 'cam.cfg'.
    /// - These files will create keybinds for you to use while TASing, unless the 'minimum_cfgs' flag is set.
    /// - Read the comment in the config file for more information on what each one does.
    /// - If the files are already present in 'cfgs/' they will be used instead of creating new ones.
    Install {
        #[clap(long)]
        projects_dir: Option<PathBuf>,
        #[clap(long)]
        half_life_dir: Option<PathBuf>,
        #[clap(long)]
        minimum_cfgs: bool,
    },
    /// Create a new project.
    ///
    /// - The project is created in the 'tas' directory, but can be changed through the config file.
    New {
        project_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        #[clap(long, conflicts_with = "no-init-git")]
        init_git: bool,
        #[clap(long)]
        no_init_git: bool,
    },
    /// Initializes a new project in an existing directory.
    ///
    /// - This is the same as 'new' but it uses an existing directory.
    Init {
        #[clap(long, short = 'n')]
        folder_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        #[clap(long, conflicts_with = "no-init-git")]
        init_git: bool,
        #[clap(long)]
        no_init_git: bool,
    },
    /// Lists all available games.
    ///
    /// - Lists all games installed in the 'Half-Life' directory.
    /// - A game is usually all directories in the Half-Life directory.
    /// - Able to set exclusions in the config file.
    Games,
    /// Runs the game.
    ///
    /// - Requires you to run from the project directory.
    RunGame {
        /// Runs the simulator client.
        #[clap(long, conflicts_with_all = &["low", "vanilla-game", "record", "width", "height", "no-bxt", "run-script"])]
        sim: bool,
        /// Runs the game with low quality settings.
        #[clap(long, conflicts_with = "record")]
        low: bool,
        /// Runs the main game with client.dll and default settings.
        #[clap(long, conflicts_with = "record")]
        vanilla_game: bool,
        /// Runs the game in high quality and 1080p resolution by default.
        #[clap(long, conflicts_with = "no-bxt")]
        record: bool,
        /// Sets the window width.
        #[clap(
            long,
            default_value("1280"),
            default_value_if("sim", None, Some("100")),
            default_value_if("record", None, Some("1920"))
        )]
        width: u32,
        /// Sets the window height.
        #[clap(
            long,
            default_value("800"),
            default_value_if("sim", None, Some("100")),
            default_value_if("record", None, Some("1080"))
        )]
        height: u32,
        /// Runs the game without bxt.
        #[clap(long, conflicts_with = "run-script")]
        no_bxt: bool,
        /// The game will run a hltas script as it starts.
        ///
        /// Useful in running the script with the 'seed' property to specify rng.
        #[clap(long)]
        run_script: Option<String>,
        /// Parameters to pass to hl.exe on start.
        #[clap(long, short)]
        params: Option<Vec<String>>,
        /// If using r-input.
        #[clap(long)]
        r_input: bool,
        /// If disabling TASView.
        #[clap(long)]
        no_tas_view: bool,
        /// Overrides the game to launch over project config.
        #[clap(long, short)]
        game_override: Option<String>,
    },
    /// Links all .hltas files to the game directory.
    ///
    /// - This command works on running from the project dir or the root dir.
    Link,
    /// Syncs the SAVE directory with the primary and secondary game directories.
    ///
    /// - This command will fail if you don't have no-client-dll-dir set in the config file.
    /// - It will copy the missing save files from each other.
    /// - If the save files are both present, it will copy the latest created one to the other.
    SyncSaves,
}
