use clap::Parser;
use goldpath::cli::Cli;
use goldpath::{run, ui};

fn main() {
    if let Err(err) = run(Cli::parse()) {
        ui::print_error(&err);
        std::process::exit(err.exit_code());
    }
}
