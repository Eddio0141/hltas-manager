use clap::StructOpt;
use hltas_manager::{run, Cli};

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
