// use ansi_term::Colour::Red;
use clap::StructOpt;
use env_logger::fmt::Color;
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
    let Cli {
        command: _,
        quiet,
        no_colour,
    } = *cli;

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

    builder.format(move |buf, record| {
        let mut style = buf.style();

        if !no_colour {
            match record.level() {
                log::Level::Error => style.set_color(Color::Red).set_bold(true),
                log::Level::Warn => style.set_color(Color::Yellow),
                log::Level::Info => style.set_color(Color::Green),
                log::Level::Debug => style.set_color(Color::Blue),
                log::Level::Trace => style.set_color(Color::White),
            };
        }

        writeln!(buf, "{}: {}", style.value(record.level()), record.args())
    });

    builder.init();
}
