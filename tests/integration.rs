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

// ─── New Fixture & Hardening Integration Tests ─────────────────────────────

#[test]
fn ruby_bundler_spec_and_rubocop() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("ruby-bundler"))
        .assert()
        .success()
        .stderr(predicate::str::contains("bundle exec rspec"));

    sgl()
        .args(["--explain", "lint", "--cwd"])
        .arg(fixtures().join("ruby-bundler"))
        .assert()
        .success()
        .stderr(predicate::str::contains("bundle exec rubocop"));
}

#[test]
fn dotnet_build_and_test() {
    sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(fixtures().join("dotnet"))
        .assert()
        .success()
        .stderr(predicate::str::contains("dotnet build"));

    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("dotnet"))
        .assert()
        .success()
        .stderr(predicate::str::contains("dotnet test"));
}

#[test]
fn python_django_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("python-django"))
        .assert()
        .success()
        .stderr(predicate::str::contains("manage.py runserver"));
}

#[test]
fn python_fastapi_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("python-fastapi"))
        .assert()
        .success()
        .stderr(predicate::str::contains("uvicorn"));
}

#[test]
fn go_with_makefile_precedence() {
    // Standard verb test is handled by Go detector
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("go-with-makefile"))
        .assert()
        .success()
        .stderr(predicate::str::contains("go test"));

    // Custom target is dispatched to make
    sgl()
        .args(["--dry-run", "run", "custom", "--cwd"])
        .arg(fixtures().join("go-with-makefile"))
        .assert()
        .success()
        .stderr(predicate::str::contains("make custom"));
}

#[test]
fn config_precedence() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("config-override"))
        .assert()
        .success()
        .stderr(predicate::str::contains("custom-override-test"));
}

#[test]
fn broken_config_reports_file_and_line() {
    sgl()
        .args(["test", "--cwd"])
        .arg(fixtures().join("broken-config"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("singularity.toml:"));
}

#[test]
fn exit_code_passthrough_3() {
    sgl()
        .args(["test", "--cwd"])
        .arg(fixtures().join("exit-code-3"))
        .assert()
        .code(3);
}

#[test]
fn missing_tool_exits_127_with_hint() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    std::fs::write(
        dir.join("singularity.toml"),
        "[commands]\ntest = \"nonexistent_tool_xyz_987\"\n",
    )
    .unwrap();

    sgl()
        .args(["test", "--cwd"])
        .arg(dir)
        .assert()
        .code(127)
        .stderr(predicate::str::contains(
            "sgl: 'nonexistent_tool_xyz_987' not found on PATH",
        ))
        .stderr(predicate::str::contains(
            "needed for: nonexistent_tool_xyz_987",
        ));
}

#[cfg(windows)]
#[test]
fn windows_cmd_shim_execution() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    let shim = dir.join("my-shim.cmd");
    std::fs::write(&shim, "@echo off\r\necho SHIM_OUTPUT\r\nexit 0\r\n").unwrap();
    std::fs::write(
        dir.join("singularity.toml"),
        "[commands]\nbuild = \"./my-shim.cmd\"\n",
    )
    .unwrap();

    sgl().args(["build", "--cwd"]).arg(dir).assert().success();
}

#[test]
fn cwd_containing_spaces_works() {
    let tmp = tempfile::TempDir::new().unwrap();
    let spaced_dir = tmp.path().join("vsc save 7").join("hanx.pro-v2");
    std::fs::create_dir_all(&spaced_dir).unwrap();
    std::fs::write(
        spaced_dir.join("package.json"),
        r#"{"name":"spaced-app","scripts":{"dev":"echo DEV_OK","build":"echo BUILD_OK"}}"#,
    )
    .unwrap();

    // 1. info works with spaced path
    sgl()
        .args(["info", "--cwd"])
        .arg(&spaced_dir)
        .assert()
        .success();

    // 2. doctor works with spaced path
    sgl()
        .args(["doctor", "--cwd"])
        .arg(&spaced_dir)
        .assert()
        .success();

    // 3. explain works with spaced path
    sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(&spaced_dir)
        .assert()
        .success()
        .stderr(predicate::str::contains("cwd:"))
        .stderr(predicate::str::contains("vsc save 7"));
}

#[test]
fn spaced_monorepo_and_gitignore() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("vsc save 7").join("hanx.pro-v2");
    let pkg_a = root.join("packages").join("pkg a");
    let ignored_pkg = root.join("packages").join("ignored pkg");
    std::fs::create_dir_all(&pkg_a).unwrap();
    std::fs::create_dir_all(&ignored_pkg).unwrap();

    std::fs::write(
        root.join("package.json"),
        r#"{"name":"root","workspaces":["packages/*"]}"#,
    )
    .unwrap();
    std::fs::write(root.join(".gitignore"), "packages/ignored pkg/\n").unwrap();
    std::fs::write(
        pkg_a.join("package.json"),
        r#"{"name":"pkg-a","scripts":{"dev":"echo DEV"}}"#,
    )
    .unwrap();
    std::fs::write(
        ignored_pkg.join("package.json"),
        r#"{"name":"ignored-pkg","scripts":{"dev":"echo DEV"}}"#,
    )
    .unwrap();

    let out = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("pkg-a"));
    assert!(!stdout.contains("ignored-pkg"));
}

#[test]
fn node_missing_script_error_message() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    std::fs::write(
        dir.join("package.json"),
        r#"{"name":"partial-app","scripts":{"dev":"vite","build":"vite build"}}"#,
    )
    .unwrap();

    sgl()
        .args(["test", "--cwd"])
        .arg(dir)
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "sgl: no 'test' command for this project (node, npm)",
        ))
        .stderr(predicate::str::contains(
            "reason: no \"test\" script in package.json",
        ))
        .stderr(predicate::str::contains("available: build, dev"))
        .stderr(predicate::str::contains("hint: add a \"test\" script to package.json, or define it in singularity.toml under [commands]"))
        .stderr(predicate::str::contains("path:"));

    // Also verify --explain test prints the exact same message
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(dir)
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "sgl: no 'test' command for this project (node, npm)",
        ))
        .stderr(predicate::str::contains(
            "reason: no \"test\" script in package.json",
        ))
        .stderr(predicate::str::contains("available: build, dev"))
        .stderr(predicate::str::contains("path:"));
}

#[test]
fn docker_lint_unavailable_error_message() {
    sgl()
        .args(["lint", "--cwd"])
        .arg(fixtures().join("compose"))
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "sgl: no 'lint' command for this project (docker)",
        ))
        .stderr(predicate::str::contains(
            "reason: docker compose does not define a standard 'lint' command",
        ))
        .stderr(predicate::str::contains("available: setup, dev, build"))
        .stderr(predicate::str::contains("path:"));
}

#[test]
fn maven_lint_unavailable_error_message() {
    sgl()
        .args(["lint", "--cwd"])
        .arg(fixtures().join("java-maven"))
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "sgl: no 'lint' command for this project (java, maven)",
        ))
        .stderr(predicate::str::contains(
            "reason: neither checkstyle nor spotless plugins are configured in pom.xml",
        ))
        .stderr(predicate::str::contains(
            "available: setup, dev, test, build",
        ))
        .stderr(predicate::str::contains("path:"));
}

#[test]
fn node_workspace_discovery() {
    let out = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(fixtures().join("node-workspace"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["schema_version"], 1);
    let projects = v["projects"].as_array().unwrap();
    assert!(projects.len() >= 2);
}

#[test]
fn rust_workspace_discovery() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("rust-workspace"))
        .assert()
        .success()
        .stderr(predicate::str::contains("cargo test"));
}

#[test]
fn monorepo_project_filter() {
    sgl()
        .args(["--dry-run", "test", "--project", "web", "--cwd"])
        .arg(fixtures().join("monorepo-mixed"))
        .assert()
        .success();
}

#[test]
fn monorepo_serial_execution() {
    sgl()
        .args(["--dry-run", "test", "--serial", "--cwd"])
        .arg(fixtures().join("monorepo-mixed"))
        .assert()
        .success();
}

#[test]
fn monorepo_changed_filtering() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();

    // Create sub-projects proj_a and proj_b
    let proj_a = root.join("proj_a");
    std::fs::create_dir_all(&proj_a).unwrap();
    std::fs::write(
        proj_a.join("package.json"),
        r#"{"name":"proj_a","scripts":{"test":"echo test-a"}}"#,
    )
    .unwrap();

    let proj_b = root.join("proj_b");
    std::fs::create_dir_all(&proj_b).unwrap();
    std::fs::write(
        proj_b.join("package.json"),
        r#"{"name":"proj_b","scripts":{"test":"echo test-b"}}"#,
    )
    .unwrap();

    // Commit both
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(root)
        .output()
        .unwrap();

    // Now modify only proj_a
    std::fs::write(
        proj_a.join("package.json"),
        r#"{"name":"proj_a","scripts":{"test":"echo test-a-modified"}}"#,
    )
    .unwrap();

    // Run sgl --dry-run test --changed
    sgl()
        .args(["--dry-run", "test", "--changed", "--cwd"])
        .arg(root)
        .assert()
        .success();
}

#[test]
fn explain_json_schema_version() {
    let out = sgl()
        .args(["--explain", "test", "--json", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON from sgl --explain --json");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["verb"], "test");
    assert!(v["plans"].as_array().is_some());
}

#[test]
fn doctor_json_schema_version() {
    let out = sgl()
        .args(["doctor", "--json", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON from sgl doctor --json");
    assert_eq!(v["schema_version"], 1);
    assert!(v["tools"].as_array().is_some());
}
