use clap::Parser;
use dev_scaffold::cli::Cli;
use dev_scaffold::{run, ui};

fn main() {
    if let Err(err) = run(Cli::parse()) {
        ui::print_error(&err);
        std::process::exit(err.exit_code());
    }
}
