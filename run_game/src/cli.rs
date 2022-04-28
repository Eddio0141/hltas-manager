use clap::{Args, Parser};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(long)]
    pub sim: bool,
    #[clap(long)]
    pub low: bool,
    #[clap(long)]
    pub no_vanilla: bool,
    #[clap(flatten)]
    pub record: Option<Record>,
    #[clap(long)]
    pub no_bxt: bool,
    pub run_script: Option<String>,
    pub params: Vec<String>,
}

#[derive(Debug, Args)]
pub struct Record {
    #[clap(long, short)]
    pub width: u32,
    #[clap(long, short)]
    pub height: u32,
}
