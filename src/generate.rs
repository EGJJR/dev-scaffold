use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::Command;

use minijinja::{context, Environment};
use rust_embed::RustEmbed;

use crate::catalog::{self, Template};
use crate::error::Error;
use crate::name::{self, crate_name};
use crate::ui;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

pub struct GenerateRequest {
    pub project_name: String,
    pub template_id: String,
    pub dest: PathBuf,
    pub dry_run: bool,
    pub no_git: bool,
}

pub struct GenerateResult {
    pub dest: PathBuf,
    pub template_id: String,
    pub files: Vec<String>,
    pub wrote: bool,
    pub git_initialized: bool,
}

pub fn resolve_template_id(explicit: Option<&str>) -> Result<String, Error> {
    if let Some(id) = explicit {
        catalog::require(id)?;
        return Ok(id.to_string());
    }
    if !io::stdin().is_terminal() {
        return Err(Error::TypeRequired);
    }
    prompt_template_id()
}

fn prompt_template_id() -> Result<String, Error> {
    let items: Vec<String> = catalog::TEMPLATES
        .iter()
        .map(|template| format!("{} — {}", template.id, template.summary))
        .collect();
    let index = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select template")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|err| Error::Dialog(err.to_string()))?;
    Ok(catalog::TEMPLATES[index].id.to_string())
}

pub fn generate(request: GenerateRequest) -> Result<GenerateResult, Error> {
    name::validate_project_name(&request.project_name)?;
    name::validate_output_path(&request.dest)?;
    let template = catalog::require(&request.template_id)?;

    if request.dest.exists() {
        return Err(Error::AlreadyExists(request.dest));
    }

    let render = ui::spinner("rendering golden-path templates");
    let files = match collect_files(template, &request.project_name) {
        Ok(files) => files,
        Err(err) => {
            render.finish_and_clear();
            return Err(err);
        }
    };
    ui::pause(280);
    if files.is_empty() {
        render.finish_and_clear();
        return Err(Error::EmptyTemplate(template.id.to_string()));
    }

    if request.dry_run {
        render.finish_and_clear();
        return Ok(GenerateResult {
            dest: request.dest,
            template_id: template.id.to_string(),
            files: files.into_iter().map(|(path, _)| path).collect(),
            wrote: false,
            git_initialized: false,
        });
    }

    render.finish_and_clear();
    let progress = ui::file_progress(files.len() as u64);
    for (rel, contents) in &files {
        progress.set_message(rel.clone());
        let path = request.dest.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents)?;
        progress.inc(1);
        ui::pause(ui::FILE_TICK_MS);
    }
    progress.finish_and_clear();

    let git_initialized = if request.no_git {
        false
    } else {
        init_git(&request.dest)
    };

    Ok(GenerateResult {
        dest: request.dest,
        template_id: template.id.to_string(),
        files: files.into_iter().map(|(path, _)| path).collect(),
        wrote: true,
        git_initialized,
    })
}

fn collect_files(template: &Template, project_name: &str) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let env = Environment::new();
    let year = chrono::Utc::now().format("%Y").to_string();
    let crate_name = crate_name(project_name);
    let ctx = context!(
        project_name => project_name,
        crate_name => crate_name,
        year => year
    );

    let prefix = format!("{}/", template.id);
    let mut files = Vec::new();

    for path in Templates::iter() {
        if !path.starts_with(&prefix) {
            continue;
        }
        let Some(file) = Templates::get(path.as_ref()) else {
            continue;
        };
        let rel = match dest_relative_path(template.id, path.as_ref(), &env, &ctx)? {
            Some(rel) => rel,
            None => continue,
        };
        let bytes = render_file(&file.data, &env, &ctx)?;
        files.push((rel, bytes));
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn dest_relative_path(
    template_id: &str,
    embed_path: &str,
    env: &Environment<'_>,
    ctx: &minijinja::value::Value,
) -> Result<Option<String>, Error> {
    let prefix = format!("{template_id}/");
    let Some(rest) = embed_path.strip_prefix(&prefix) else {
        return Ok(None);
    };
    if rest.is_empty() || rest.ends_with(".keep") {
        return Ok(None);
    }
    let mapped = map_dot_segments(rest);
    let rendered = env.render_str(&mapped, ctx)?;
    if rendered.is_empty() {
        return Ok(None);
    }
    Ok(Some(rendered.replace('\\', "/")))
}

pub fn map_dot_segments(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            segment
                .strip_prefix("dot.")
                .map(|rest| format!(".{rest}"))
                .unwrap_or_else(|| segment.to_string())
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn render_file(
    data: &[u8],
    env: &Environment<'_>,
    ctx: &minijinja::value::Value,
) -> Result<Vec<u8>, Error> {
    match std::str::from_utf8(data) {
        Ok(text) => {
            let mut rendered = env.render_str(text, ctx)?;
            if text.ends_with('\n') && !rendered.ends_with('\n') {
                rendered.push('\n');
            }
            Ok(rendered.into_bytes())
        }
        Err(_) => Ok(data.to_vec()),
    }
}

fn init_git(dest: &Path) -> bool {
    match Command::new("git")
        .arg("init")
        .arg("--quiet")
        .arg(dest)
        .status()
    {
        Ok(status) if status.success() => true,
        Ok(_) => {
            ui::print_warning("git init failed");
            false
        }
        Err(_) => {
            ui::print_warning("git not found; skipped git init");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::map_dot_segments;

    #[test]
    fn maps_dot_prefix_segments() {
        assert_eq!(map_dot_segments("dot.gitignore"), ".gitignore");
        assert_eq!(
            map_dot_segments("dot.github/workflows/ci.yml"),
            ".github/workflows/ci.yml"
        );
        assert_eq!(map_dot_segments("src/main.py"), "src/main.py");
    }
}
