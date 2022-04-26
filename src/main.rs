use clap::StructOpt;
use hltas_manager::{cli::Cli, run};

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}
