use clap::Parser;
// use ansi_term::Colour::Red;
use hltas_manager::{cli::Cli, commands::run};
use log::{error, LevelFilter};
use std::io::Write;

fn main() {
    let cli = Cli::parse();

    init_logger(&cli);

    if let Err(err) = run(cli) {
        error!("{:?}", err);

        std::process::exit(1);
    }
}

fn init_logger(cli: &Cli) {
    let Cli { command: _, quiet } = *cli;

    if quiet {
        return;
    }

    let mut builder = env_logger::builder();

    #[cfg(debug_assertions)]
    builder
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    builder
        .format_timestamp(None)
        .filter_level(LevelFilter::Info);

    builder.format(move |buf, record| writeln!(buf, "{}: {}", record.level(), record.args()));

    builder.init();
}
