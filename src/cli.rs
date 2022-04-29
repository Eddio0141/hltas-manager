use std::path::PathBuf;

use clap::*;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
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
    RunGame {
        #[clap(long, conflicts_with_all = &["low", "no-vanilla", "record", "width", "height", "no-bxt", "run-script"])]
        sim: bool,
        #[clap(long, conflicts_with = "record")]
        low: bool,
        #[clap(long, conflicts_with = "record")]
        vanilla_game: bool,
        #[clap(long, conflicts_with = "no-bxt")]
        record: bool,
        #[clap(
            long,
            short,
            default_value("1280"),
            default_value_if("sim", None, Some("100"))
        )]
        width: u32,
        #[clap(
            long,
            short,
            default_value("800"),
            default_value_if("sim", None, Some("100"))
        )]
        height: u32,
        #[clap(long, conflicts_with = "run-script")]
        no_bxt: bool,
        #[clap(long)]
        run_script: Option<String>,
        #[clap(long, short)]
        params: Option<Vec<String>>,
        #[clap(long)]
        r_input: bool,
        #[clap(long)]
        no_tas_view: bool,
    },
}
