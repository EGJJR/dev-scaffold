pub mod catalog;
pub mod cli;
pub mod error;
pub mod generate;
pub mod name;
pub mod ui;

use std::path::PathBuf;

use crate::cli::{Cli, Command};
use crate::error::Error;
use crate::generate::{generate, resolve_template_id, GenerateRequest};

pub fn run(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Some(Command::List) => {
            ui::print_list();
            Ok(())
        }
        None => run_init(cli),
    }
}

fn run_init(cli: Cli) -> Result<(), Error> {
    let project_name = cli.name.ok_or(Error::NameRequired)?;
    crate::name::validate_project_name(&project_name)?;

    let template_id = resolve_template_id(cli.type_name.as_deref())?;
    let dest = match cli.output {
        Some(path) => {
            crate::name::validate_output_path(&path)?;
            path
        }
        None => PathBuf::from(&project_name),
    };

    ui::print_banner();
    ui::print_job(&project_name, &template_id, cli.dry_run);

    let result = generate(GenerateRequest {
        project_name: project_name.clone(),
        template_id,
        dest,
        dry_run: cli.dry_run,
        no_git: cli.no_git,
    })?;

    let root_name = result
        .dest
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(project_name.as_str());

    if result.wrote {
        ui::print_checklist(&result);
        ui::print_tree(root_name, &result.files);
        println!();
        ui::print_next_steps(&result);
    } else {
        ui::print_tree(root_name, &result.files);
    }
    Ok(())
}
