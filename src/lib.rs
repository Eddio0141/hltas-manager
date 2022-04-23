use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Install {
        projects_dir_name: String,
    },
    New {
        #[clap(long, short = 'n')]
        project_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        // TODO depends on if game_name is set
        #[clap(long, short)]
        copy_game_dir_for_sim_client: bool,
    },
    Init {
        #[clap(long, short = 'n')]
        folder_name: String,
        #[clap(long, short)]
        game_name: Option<String>,
        // TODO depends on if game_name is set
        #[clap(long, short)]
        copy_game_dir_for_sim_client: bool,
    },
    Games,
    GenCfgs {
        #[clap(long)]
        minimum_cfgs: bool,
        #[clap(long)]
        no_userconfig_change: bool,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    dbg!(cli);
    todo!()
}
