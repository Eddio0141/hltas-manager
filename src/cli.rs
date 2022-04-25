use clap::*;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Sets up the required files for the tool.
    ///
    /// - Can be used again to verify if the files are already installed.
    /// - Requires the 'steamapi.dll' file to be the default dll.
    /// - Needs to be run before the tool can be used.
    // TODO verify if all steam_api.dll hash are the same, then remove the first doc comment line
    Install {
        #[clap(long)]
        projects_dir_name: Option<String>,
    },
    /// Create a new project.
    ///
    /// - The project is created in the 'tas' directory, but can be changed through the config file.
    New {
        #[clap(long, short = 'n')]
        project_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        // TODO depends on if game_name is set
        #[clap(long, short)]
        copy_game_dir_for_sim_client: bool,
        #[clap(long, conflicts_with = "no-init-git")]
        init_git: bool,
        #[clap(long)]
        no_init_git: bool,
        #[clap(long, conflicts_with = "no-copy-hltas-cfgs")]
        copy_hltas_cfgs: bool,
        #[clap(long)]
        no_copy_hltas_cfgs: bool,
    },
    /// Initializes a new project in an existing directory.
    ///
    /// - This is the same as 'new' but it uses an existing directory.
    Init {
        #[clap(long, short = 'n')]
        folder_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        // TODO depends on if game_name is set
        #[clap(long, short)]
        copy_game_dir_for_sim_client: bool,
    },
    /// Lists all available games.
    ///
    /// - Lists all games installed in the 'Half-Life' directory.
    /// - A game is usually all directories in the Half-Life directory.
    /// - Able to set exclusions in the config file.
    // TODO auto detect if its a game or unrelated dir
    Games,
    /// Generates the .cfg files for TASing.
    ///
    /// - Creates the cfg files 'hltas.cfg', 'ingame.cfg', 'record.cfg', 'editor.cfg' and 'cam.cfg'.
    /// - These files will create keybinds for you to use while TASing, unless the 'minimum_cfgs' flag is set.
    /// - Read the comment in the config file for more information on what each one does.
    /// - If the files are already present in 'cfgs/' they will be used instead of creating new ones.
    GenCfgs {
        #[clap(long)]
        minimum_cfgs: bool,
        #[clap(long)]
        no_userconfig_change: bool,
    },
}
