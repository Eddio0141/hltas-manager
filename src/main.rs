use ansi_term::Colour::Red;
use clap::StructOpt;
use hltas_manager::{cli::Cli, commands::run};

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        let error_text = Red.bold().paint("error:");
        eprintln!("{error_text} {:?}", err);

        std::process::exit(1);
    }
}
