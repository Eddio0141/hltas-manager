// use ansi_term::Colour::Red;
use clap::StructOpt;
use env_logger::fmt::Color;
use hltas_manager::{cli::Cli, commands::run};
use log::{error, LevelFilter};
use std::io::Write;

fn main() {
    init_logger();

    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        error!("{:?}", err);

        std::process::exit(1);
    }
}

fn init_logger() {
    let mut builder = env_logger::builder();

    builder
        .format_timestamp(None)
        .filter_level(LevelFilter::Info);

    builder.format(|buf, record| {
        let mut style = buf.style();

        match record.level() {
            log::Level::Error => style.set_color(Color::Red).set_bold(true),
            log::Level::Warn => style.set_color(Color::Yellow),
            log::Level::Info => style.set_color(Color::Green),
            log::Level::Debug => style.set_color(Color::Blue),
            log::Level::Trace => style.set_color(Color::White),
        };

        dbg!(record.level());

        writeln!(buf, "{}: {}", style.value(record.level()), record.args())
    });

    builder.init();
}
