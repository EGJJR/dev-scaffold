use std::path::PathBuf;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};

fn clap_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Cyan.on_default())
}

#[derive(Debug, Parser)]
#[command(
    name = "dev-scaffold",
    about = "Scaffold a production-ready service from a secure template",
    styles = clap_styles(),
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Name of the service directory to create (kebab-case)
    pub name: Option<String>,

    /// Template: api, api-rust, or worker
    #[arg(short = 't', long = "type")]
    pub type_name: Option<String>,

    /// Destination directory (defaults to ./<name>)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Print the file tree without writing files
    #[arg(long)]
    pub dry_run: bool,

    /// Skip git init
    #[arg(long)]
    pub no_git: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List available templates
    List,
}
