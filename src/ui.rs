use std::collections::BTreeMap;
use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::Duration;

use console::{style, Color, Style};
use indicatif::{ProgressBar, ProgressStyle};

use crate::catalog;
use crate::generate::GenerateResult;

pub const FILE_TICK_MS: u64 = 18;

pub fn animated() -> bool {
    io::stdout().is_terminal()
        && io::stderr().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
}

pub fn pause(ms: u64) {
    if animated() {
        thread::sleep(Duration::from_millis(ms));
    }
}

pub fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "  {}  {}  {}",
        style("◆").cyan().bold(),
        style("GOLDPATH").white().bold(),
        style(format!("v{version}")).dim()
    );
    println!(
        "     {}",
        style("golden-path scaffolding · secure defaults").dim()
    );
    println!(
        "  {}",
        style("────────────────────────────────────────").cyan()
    );
    println!();
}

pub fn print_job(project: &str, template: &str, dry_run: bool) {
    let kind = if dry_run { "Dry run" } else { "scaffold" };
    let kind_style = if dry_run {
        style(format!("{kind:<8}")).yellow().bold()
    } else {
        style(format!("{kind:<8}")).cyan().bold()
    };
    println!("  {kind_style}  {}", style(project).white().bold());
    println!(
        "  {}  {}",
        style(format!("{:<8}", "template")).dim(),
        style(template).green()
    );
    println!();
}

pub fn spinner(message: &str) -> ProgressBar {
    if !animated() {
        return ProgressBar::hidden();
    }
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::with_template("  {spinner:.cyan}  {msg}")
            .expect("spinner template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
    );
    bar.set_message(message.to_string());
    bar.enable_steady_tick(Duration::from_millis(70));
    bar
}

pub fn file_progress(total: u64) -> ProgressBar {
    if !animated() {
        return ProgressBar::hidden();
    }
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan}  {msg:<32} [{bar:20.cyan/blue}] {pos}/{len}",
        )
        .expect("bar template")
        .progress_chars("━╾─"),
    );
    bar
}

pub fn print_checklist(result: &GenerateResult) {
    let git_label = if result.git_initialized {
        "git repository"
    } else {
        "git skipped"
    };
    let mut items = vec![
        "render templates".to_string(),
        "non-root pinned container".to_string(),
        "ci audit + least privilege".to_string(),
        git_label.to_string(),
    ];
    if matches!(result.template_id.as_str(), "api" | "worker") {
        items.push("uv package manager".into());
    }
    for label in items {
        pause(55);
        println!("  {}  {label}", style("✔").green().bold());
        let _ = io::stdout().flush();
    }
    pause(55);
    println!(
        "  {}  write {} files",
        style("✔").green().bold(),
        result.files.len()
    );
    println!();
}

pub fn print_tree(root_name: &str, files: &[String]) {
    let cyan = Style::new().fg(Color::Cyan).bold();
    println!("  {}", cyan.apply_to(format!("{root_name}/")));
    render_dir(&build_tree(files), "  ");
}

pub fn print_next_steps(result: &GenerateResult) {
    let dir = result
        .dest
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(".");
    println!("  {}", style("Next").white().bold());
    println!("  {}", style("────").dim());
    println!("    {} {dir}", style("cd").cyan());
    for line in next_commands(&result.template_id) {
        println!("    {}", style(line).cyan());
    }
    println!();
    println!(
        "  {}",
        style("A justfile is included if you use just. Install uv: https://docs.astral.sh/uv/")
            .dim()
    );
}

fn next_commands(template_id: &str) -> Vec<&'static str> {
    match template_id {
        "api-rust" => vec![
            "SECRET_KEY=dev-only-not-for-production cargo test",
            "SECRET_KEY=dev-only-not-for-production cargo run",
        ],
        "worker" => vec![
            "uv sync --extra dev",
            "SECRET_KEY=dev-only-not-for-production ENV=test uv run pytest",
            "SECRET_KEY=dev-only-not-for-production uv run python -m app.main",
        ],
        _ => vec![
            "uv sync --extra dev",
            "SECRET_KEY=dev-only-not-for-production ENV=test uv run pytest",
            "SECRET_KEY=dev-only-not-for-production ENV=development uv run uvicorn app.main:app --reload --host 127.0.0.1 --port 8000",
        ],
    }
}

pub fn print_list() {
    print_banner();
    println!("  {}", style("templates").white().bold());
    println!("  {}", style("─────────").dim());
    let width = catalog::TEMPLATES
        .iter()
        .map(|template| template.id.len())
        .max()
        .unwrap_or(0);
    for template in catalog::TEMPLATES {
        println!(
            "  {}  {}",
            style(format!("{:width$}", template.id, width = width))
                .green()
                .bold(),
            style(template.summary).dim()
        );
    }
    println!();
}

pub fn print_error(err: &impl std::fmt::Display) {
    eprintln!("{} {err}", style("error").red().bold());
}

pub fn print_warning(message: &str) {
    eprintln!("{} {message}", style("warning").yellow().bold());
}

fn build_tree(files: &[String]) -> Dir {
    let mut root = Dir::default();
    for file in files {
        root.insert(file);
    }
    root
}

#[derive(Default)]
struct Dir {
    dirs: BTreeMap<String, Dir>,
    files: BTreeMap<String, ()>,
}

impl Dir {
    fn insert(&mut self, path: &str) {
        let mut parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
        if parts.is_empty() {
            return;
        }
        let file = parts.pop().unwrap().to_string();
        let mut node = self;
        for part in parts {
            node = node.dirs.entry(part.to_string()).or_default();
        }
        node.files.insert(file, ());
    }

    fn entries(&self) -> Vec<Entry<'_>> {
        let mut entries = Vec::new();
        for (name, dir) in &self.dirs {
            entries.push(Entry::Dir(name, dir));
        }
        for name in self.files.keys() {
            entries.push(Entry::File(name));
        }
        entries
    }
}

enum Entry<'a> {
    Dir(&'a str, &'a Dir),
    File(&'a str),
}

fn render_dir(dir: &Dir, prefix: &str) {
    let entries = dir.entries();
    let last_index = entries.len().saturating_sub(1);
    for (index, entry) in entries.into_iter().enumerate() {
        let last = index == last_index;
        let branch = if last { "└── " } else { "├── " };
        let child_prefix = if last { "    " } else { "│   " };
        match entry {
            Entry::Dir(name, child) => {
                println!(
                    "{prefix}{}{}",
                    style(branch).dim(),
                    style(format!("{name}/")).cyan()
                );
                render_dir(child, &format!("{prefix}{child_prefix}"));
            }
            Entry::File(name) => {
                println!("{prefix}{}{name}", style(branch).dim());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::build_tree;

    #[test]
    fn nests_directories() {
        let tree = build_tree(&[
            ".github/workflows/ci.yml".into(),
            "Dockerfile".into(),
            "app/main.py".into(),
        ]);
        assert!(tree.files.contains_key("Dockerfile"));
        assert!(tree.dirs.contains_key(".github"));
        assert!(tree.dirs.contains_key("app"));
    }
}
