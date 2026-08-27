use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::str;
use tempfile::TempDir;

fn bin() -> Command {
    Command::cargo_bin("dev-scaffold").expect("binary")
}

fn scaffold(tmp: &TempDir, name: &str, type_name: &str) -> assert_cmd::assert::Assert {
    bin()
        .current_dir(tmp.path())
        .args([name, "--type", type_name, "--no-git"])
        .assert()
        .success()
}

fn read(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|err| panic!("read {rel}: {err}"))
}

#[test]
fn list_prints_all_templates() {
    bin()
        .arg("list")
        .assert()
        .success()
        .stdout(str::contains("api"))
        .stdout(str::contains("api-rust"))
        .stdout(str::contains("worker"));
}

#[test]
fn rejects_missing_name() {
    bin()
        .assert()
        .failure()
        .code(2)
        .stderr(str::contains("project name is required"));
}

#[test]
fn rejects_unknown_type() {
    bin()
        .args(["payment-service", "--type", "java"])
        .assert()
        .failure()
        .code(2)
        .stderr(str::contains("unknown template"));
}

#[test]
fn rejects_path_traversal_name() {
    bin()
        .args(["../evil", "--type", "api"])
        .assert()
        .failure()
        .code(2)
        .stderr(str::contains("path"));
}

#[test]
fn rejects_nested_name() {
    bin()
        .args(["foo/bar", "--type", "api"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn rejects_existing_directory() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join("payment-service")).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["payment-service", "--type", "api", "--no-git"])
        .assert()
        .failure()
        .stderr(str::contains("already exists"));
}

#[test]
fn dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["payment-service", "--type", "api", "--dry-run"])
        .assert()
        .success()
        .stdout(str::contains("Dry run"))
        .stdout(str::contains("Dockerfile"));
    assert!(!tmp.path().join("payment-service").exists());
}

#[test]
fn generates_api_with_secure_defaults() {
    let tmp = TempDir::new().unwrap();
    scaffold(&tmp, "payment-service", "api");
    let root = tmp.path().join("payment-service");

    assert!(root.join(".github/workflows/ci.yml").exists());
    assert!(root.join(".github/dependabot.yml").exists());
    assert!(root.join("app/main.py").exists());

    let gitignore = read(&root, ".gitignore");
    assert!(gitignore.contains(".env"));

    let dockerfile = read(&root, "Dockerfile");
    assert!(dockerfile.contains("USER app"));
    assert!(!dockerfile.contains(":latest"));
    assert!(dockerfile.contains("python:3.12-slim-bookworm"));
    assert!(dockerfile.contains("astral-sh/uv"));

    let ci = read(&root, ".github/workflows/ci.yml");
    assert!(ci.contains("permissions:"));
    assert!(ci.contains("contents: read"));
    assert!(ci.contains("pip-audit"));
    assert!(ci.contains("astral-sh/setup-uv"));
    assert!(ci.contains("uv sync"));
    assert!(ci.contains("docker build"));

    assert!(root.join(".python-version").exists());

    let main = read(&root, "app/main.py");
    assert!(main.contains("is_production()"));
    assert!(main.contains("docs_url"));
    assert!(main.contains("payment-service"));

    let env_example = read(&root, ".env.example");
    assert!(env_example.contains("SECRET_KEY=change-me"));
    assert_no_live_secrets(&root);
}

#[test]
fn next_steps_use_native_commands() {
    let tmp = TempDir::new().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["payment-service", "--type", "api", "--no-git"])
        .assert()
        .success()
        .stdout(str::contains("uv sync"))
        .stdout(str::contains("uv run pytest"))
        .stdout(str::contains("uv run uvicorn"));
}

#[test]
fn generates_worker_and_rust_templates() {
    let tmp = TempDir::new().unwrap();
    scaffold(&tmp, "billing-worker", "worker");
    scaffold(&tmp, "edge-api", "api-rust");

    let worker_docker = read(&tmp.path().join("billing-worker"), "Dockerfile");
    assert!(worker_docker.contains("USER app"));
    assert!(!worker_docker.contains(":latest"));

    let rust_ci = read(&tmp.path().join("edge-api"), ".github/workflows/ci.yml");
    assert!(rust_ci.contains("cargo audit"));
    assert!(rust_ci.contains("contents: read"));

    let rust_main = read(&tmp.path().join("edge-api"), "src/main.rs");
    assert!(rust_main.contains("use edge_api::app"));
    assert!(rust_main.contains("SECRET_KEY"));

    assert_no_live_secrets(&tmp.path().join("billing-worker"));
    assert_no_live_secrets(&tmp.path().join("edge-api"));
}

#[test]
fn substitutes_project_name_in_tree() {
    let tmp = TempDir::new().unwrap();
    scaffold(&tmp, "checkout-api", "api");
    let readme = read(&tmp.path().join("checkout-api"), "README.md");
    assert!(readme.contains("checkout-api"));
    assert!(!readme.contains("{{ project_name }}"));
}

fn assert_no_live_secrets(root: &Path) {
    visit_files(root, &mut |path, contents| {
        assert!(
            !contents.contains("{{ project_name }}"),
            "unrendered template in {}",
            path.display()
        );
        assert!(
            !contents.contains("{{ crate_name }}"),
            "unrendered template in {}",
            path.display()
        );
        for line in contents.lines() {
            let lower = line.to_ascii_lowercase();
            if lower.contains("secret_key=") {
                let allowed = lower.contains("change-me")
                    || lower.contains("test-only")
                    || lower.contains("dev-only")
                    || lower.contains("${secret_key")
                    || lower.contains("alias=")
                    || lower.contains("field(")
                    || lower.contains("env::var")
                    || lower.contains("is_err")
                    || lower.contains("is missing")
                    || lower.contains("not-secret");
                assert!(
                    allowed,
                    "unexpected SECRET_KEY assignment in {}: {}",
                    path.display(),
                    line
                );
            }
            assert!(
                !line.contains("AKIA"),
                "looks like an AWS key in {}",
                path.display()
            );
        }
    });
}

fn visit_files(root: &Path, visit: &mut impl FnMut(&Path, &str)) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            visit_files(&path, visit);
        } else if let Ok(contents) = fs::read_to_string(&path) {
            visit(&path, &contents);
        }
    }
}
