//! Integration tests: run `sgl --explain <verb>` against each fixture.
//!
//! --explain prints to stderr. We check stderr with predicates::str::contains.
//! --dry-run is used so no real toolchains are needed.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn sgl() -> Command {
    Command::cargo_bin("sgl").unwrap()
}

fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

// ─── Node ──────────────────────────────────────────────────────────────────

#[test]
fn node_npm_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("node-npm"))
        .assert()
        .success()
        .stderr(predicate::str::contains("npm run dev"));
}

#[test]
fn node_npm_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("node-npm"))
        .assert()
        .success()
        .stderr(predicate::str::contains("npm run test"));
}

#[test]
fn node_npm_build() {
    sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(fixtures().join("node-npm"))
        .assert()
        .success()
        .stderr(predicate::str::contains("npm run build"));
}

#[test]
fn node_npm_fmt() {
    sgl()
        .args(["--explain", "fmt", "--cwd"])
        .arg(fixtures().join("node-npm"))
        .assert()
        .success()
        .stderr(predicate::str::contains("npm run format"));
}

#[test]
fn node_pnpm_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("node-pnpm"))
        .assert()
        .success()
        .stderr(predicate::str::contains("pnpm run dev"));
}

#[test]
fn node_yarn_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("node-yarn"))
        .assert()
        .success()
        .stderr(predicate::str::contains("yarn test"));
}

#[test]
fn node_bun_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("node-bun"))
        .assert()
        .success()
        .stderr(predicate::str::contains("bun run dev"));
}

// ─── Python ────────────────────────────────────────────────────────────────

#[test]
fn python_uv_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("python-uv"))
        .assert()
        .success()
        .stderr(predicate::str::contains("uv"));
}

#[test]
fn python_uv_setup() {
    sgl()
        .args(["--explain", "setup", "--cwd"])
        .arg(fixtures().join("python-uv"))
        .assert()
        .success()
        .stderr(predicate::str::contains("uv sync"));
}

#[test]
fn python_poetry_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("python-poetry"))
        .assert()
        .success()
        .stderr(predicate::str::contains("poetry"));
}

#[test]
fn python_pip_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("python-pip"))
        .assert()
        .success()
        .stderr(predicate::str::contains("pytest"));
}

// ─── Rust ──────────────────────────────────────────────────────────────────

#[test]
fn rust_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("rust"))
        .assert()
        .success()
        .stderr(predicate::str::contains("cargo test"));
}

#[test]
fn rust_lint() {
    sgl()
        .args(["--explain", "lint", "--cwd"])
        .arg(fixtures().join("rust"))
        .assert()
        .success()
        .stderr(predicate::str::contains("cargo clippy"));
}

#[test]
fn rust_fmt() {
    sgl()
        .args(["--explain", "fmt", "--cwd"])
        .arg(fixtures().join("rust"))
        .assert()
        .success()
        .stderr(predicate::str::contains("cargo fmt"));
}

// ─── Go ────────────────────────────────────────────────────────────────────

#[test]
fn go_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("go"))
        .assert()
        .success()
        .stderr(predicate::str::contains("go test ./..."));
}

#[test]
fn go_fmt() {
    sgl()
        .args(["--explain", "fmt", "--cwd"])
        .arg(fixtures().join("go"))
        .assert()
        .success()
        .stderr(predicate::str::contains("gofmt"));
}

// ─── Make ──────────────────────────────────────────────────────────────────

#[test]
fn make_run_target() {
    // Use --dry-run with the actual `run` subcommand and check stderr for the command.
    // Note: --explain only accepts a single verb keyword; for `run <target>` use dry-run.
    sgl()
        .args(["--dry-run", "run", "build", "--cwd"])
        .arg(fixtures().join("make"))
        .assert()
        .success()
        .stderr(predicate::str::contains("make build"));
}

#[test]
fn make_test_verb() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("make"))
        .assert()
        .success()
        .stderr(predicate::str::contains("make test"));
}

// ─── Docker Compose ────────────────────────────────────────────────────────

#[test]
fn compose_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("compose"))
        .assert()
        .success()
        .stderr(predicate::str::contains("docker compose up"));
}

#[test]
fn compose_build() {
    sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(fixtures().join("compose"))
        .assert()
        .success()
        .stderr(predicate::str::contains("docker compose build"));
}

// ─── Java Maven ────────────────────────────────────────────────────────────

#[test]
fn java_maven_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("java-maven"))
        .assert()
        .success()
        .stderr(predicate::str::contains("mvn test"));
}

// ─── Monorepo discovery ────────────────────────────────────────────────────

#[test]
fn monorepo_info_json_lists_projects() {
    let out = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(fixtures().join("monorepo-mixed"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON from sgl info --json");
    let projects = v["projects"].as_array().expect("expected projects array");
    assert!(
        projects.len() >= 2,
        "expected at least 2 projects, got: {stdout}"
    );
}

// ─── Config override precedence ────────────────────────────────────────────

#[test]
fn config_override_wins() {
    use std::fs;
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();

    fs::write(dir.join("requirements.txt"), "pytest").unwrap();
    fs::write(
        dir.join("singularity.toml"),
        "[commands]\ntest = \"pytest -x --tb=short\"\n",
    )
    .unwrap();

    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(dir)
        .assert()
        .success()
        .stderr(predicate::str::contains("pytest -x --tb=short"));
}

// ─── --dry-run exit zero ───────────────────────────────────────────────────

#[test]
fn dry_run_exits_zero() {
    sgl()
        .args(["--dry-run", "test", "--cwd"])
        .arg(fixtures().join("rust"))
        .assert()
        .success();
}

// ─── sgl info --json output ────────────────────────────────────────────────

#[test]
fn info_json_valid() {
    let out = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON from sgl info --json");
    assert!(v.get("projects").is_some());
}

// ─── completions ───────────────────────────────────────────────────────────

#[test]
fn completions_bash() {
    sgl()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sgl"));
}

#[test]
fn completions_powershell() {
    sgl()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sgl"));
}
