//! Integration tests for singularity-cli.
//!
//! --explain output (command line and plan details) goes to stdout.
//! Errors go to stderr.
//! --dry-run "→" lines go to stdout.

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
        .stdout(predicate::str::contains("npm run dev"));
}

#[test]
fn node_npm_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("node-npm"))
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run test"));
}

#[test]
fn node_npm_build() {
    sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(fixtures().join("node-npm"))
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run build"));
}

#[test]
fn node_npm_fmt() {
    sgl()
        .args(["--explain", "fmt", "--cwd"])
        .arg(fixtures().join("node-npm"))
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run format"));
}

#[test]
fn node_pnpm_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("node-pnpm"))
        .assert()
        .success()
        .stdout(predicate::str::contains("pnpm run dev"));
}

#[test]
fn node_yarn_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("node-yarn"))
        .assert()
        .success()
        .stdout(predicate::str::contains("yarn test"));
}

#[test]
fn node_bun_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("node-bun"))
        .assert()
        .success()
        .stdout(predicate::str::contains("bun run dev"));
}

// ─── Python ────────────────────────────────────────────────────────────────

#[test]
fn python_uv_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("python-uv"))
        .assert()
        .success()
        .stdout(predicate::str::contains("uv"));
}

#[test]
fn python_uv_setup() {
    sgl()
        .args(["--explain", "setup", "--cwd"])
        .arg(fixtures().join("python-uv"))
        .assert()
        .success()
        .stdout(predicate::str::contains("uv sync"));
}

#[test]
fn python_poetry_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("python-poetry"))
        .assert()
        .success()
        .stdout(predicate::str::contains("poetry"));
}

#[test]
fn python_pip_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("python-pip"))
        .assert()
        .success()
        .stdout(predicate::str::contains("pytest"));
}

// ─── Rust ──────────────────────────────────────────────────────────────────

#[test]
fn rust_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("rust"))
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo test"));
}

#[test]
fn rust_lint() {
    sgl()
        .args(["--explain", "lint", "--cwd"])
        .arg(fixtures().join("rust"))
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo clippy"));
}

#[test]
fn rust_fmt() {
    sgl()
        .args(["--explain", "fmt", "--cwd"])
        .arg(fixtures().join("rust"))
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo fmt"));
}

// ─── Go ────────────────────────────────────────────────────────────────────

#[test]
fn go_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("go"))
        .assert()
        .success()
        .stdout(predicate::str::contains("go test ./..."));
}

#[test]
fn go_fmt() {
    sgl()
        .args(["--explain", "fmt", "--cwd"])
        .arg(fixtures().join("go"))
        .assert()
        .success()
        .stdout(predicate::str::contains("gofmt"));
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
        .stdout(predicate::str::contains("make build"));
}

#[test]
fn make_test_verb() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("make"))
        .assert()
        .success()
        .stdout(predicate::str::contains("make test"));
}

// ─── Docker Compose ────────────────────────────────────────────────────────

#[test]
fn compose_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("compose"))
        .assert()
        .success()
        .stdout(predicate::str::contains("docker compose up"));
}

#[test]
fn compose_build() {
    sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(fixtures().join("compose"))
        .assert()
        .success()
        .stdout(predicate::str::contains("docker compose build"));
}

// ─── Java Maven ────────────────────────────────────────────────────────────

#[test]
fn java_maven_test() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("java-maven"))
        .assert()
        .success()
        .stdout(predicate::str::contains("mvn test"));
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
        .stdout(predicate::str::contains("pytest -x --tb=short"));
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
        .stdout(predicate::str::contains("bundle exec rspec"));

    sgl()
        .args(["--explain", "lint", "--cwd"])
        .arg(fixtures().join("ruby-bundler"))
        .assert()
        .success()
        .stdout(predicate::str::contains("bundle exec rubocop"));
}

#[test]
fn dotnet_build_and_test() {
    sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(fixtures().join("dotnet"))
        .assert()
        .success()
        .stdout(predicate::str::contains("dotnet build"));

    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("dotnet"))
        .assert()
        .success()
        .stdout(predicate::str::contains("dotnet test"));
}

#[test]
fn python_django_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("python-django"))
        .assert()
        .success()
        .stdout(predicate::str::contains("manage.py runserver"));
}

#[test]
fn python_fastapi_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("python-fastapi"))
        .assert()
        .success()
        .stdout(predicate::str::contains("uvicorn"));
}

#[test]
fn go_with_makefile_precedence() {
    // Standard verb test is handled by Go detector
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("go-with-makefile"))
        .assert()
        .success()
        .stdout(predicate::str::contains("go test"));

    // Custom target is dispatched to make
    sgl()
        .args(["--dry-run", "run", "custom", "--cwd"])
        .arg(fixtures().join("go-with-makefile"))
        .assert()
        .success()
        .stdout(predicate::str::contains("make custom"));
}

#[test]
fn config_precedence() {
    sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("config-override"))
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-override-test"));
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

#[cfg(windows)]
#[test]
fn windows_ps1_and_cmd_on_path_with_spaces() {
    let old_path = std::env::var_os("PATH").unwrap_or_default();

    // 1. Temp dir with ONLY fake.ps1
    let tmp_bin_ps1 = tempfile::TempDir::new().unwrap();
    let ps1_file = tmp_bin_ps1.path().join("fake.ps1");
    std::fs::write(&ps1_file, "Write-Output 'FAKE_PS1_OK'\r\nexit 0\r\n").unwrap();

    let tmp_proj_ps1 = tempfile::TempDir::new().unwrap();
    let spaced_cwd_ps1 = tmp_proj_ps1
        .path()
        .join("dir with spaces")
        .join("my proj ps1");
    std::fs::create_dir_all(&spaced_cwd_ps1).unwrap();
    std::fs::write(
        spaced_cwd_ps1.join("singularity.toml"),
        "[commands]\ntest = \"fake\"\n",
    )
    .unwrap();

    let mut path_with_ps1 = tmp_bin_ps1.path().as_os_str().to_os_string();
    path_with_ps1.push(";");
    path_with_ps1.push(&old_path);

    sgl()
        .env("PATH", &path_with_ps1)
        .args(["test", "--cwd"])
        .arg(&spaced_cwd_ps1)
        .assert()
        .success();

    // 2. Temp dir with ONLY fake.cmd
    let tmp_bin_cmd = tempfile::TempDir::new().unwrap();
    let cmd_file = tmp_bin_cmd.path().join("fake.cmd");
    std::fs::write(&cmd_file, "@echo off\r\necho FAKE_CMD_OK\r\nexit 0\r\n").unwrap();

    let tmp_proj_cmd = tempfile::TempDir::new().unwrap();
    let spaced_cwd_cmd = tmp_proj_cmd
        .path()
        .join("dir with spaces")
        .join("my proj cmd");
    std::fs::create_dir_all(&spaced_cwd_cmd).unwrap();
    std::fs::write(
        spaced_cwd_cmd.join("singularity.toml"),
        "[commands]\nbuild = \"fake\"\n",
    )
    .unwrap();

    let mut path_with_cmd = tmp_bin_cmd.path().as_os_str().to_os_string();
    path_with_cmd.push(";");
    path_with_cmd.push(&old_path);

    sgl()
        .env("PATH", &path_with_cmd)
        .args(["build", "--cwd"])
        .arg(&spaced_cwd_cmd)
        .assert()
        .success();

    // 3. Resolution order: cmd preferred over ps1 when both exist next to each other
    let tmp_bin_both = tempfile::TempDir::new().unwrap();
    std::fs::write(
        tmp_bin_both.path().join("tool.cmd"),
        "@echo off\r\necho RUNNING_CMD\r\nexit 0\r\n",
    )
    .unwrap();
    std::fs::write(
        tmp_bin_both.path().join("tool.ps1"),
        "Write-Output 'RUNNING_PS1'\r\nexit 0\r\n",
    )
    .unwrap();

    let tmp_proj_both = tempfile::TempDir::new().unwrap();
    let cwd_both = tmp_proj_both.path().join("proj");
    std::fs::create_dir_all(&cwd_both).unwrap();
    std::fs::write(
        cwd_both.join("singularity.toml"),
        "[commands]\nrun = \"tool\"\n",
    )
    .unwrap();

    let mut path_with_both = tmp_bin_both.path().as_os_str().to_os_string();
    path_with_both.push(";");
    path_with_both.push(&old_path);

    sgl()
        .env("PATH", &path_with_both)
        .args(["run", "run", "--cwd"])
        .arg(&cwd_both)
        .assert()
        .success()
        .stdout(predicate::str::contains("RUNNING_CMD"));
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
        .stdout(predicate::str::contains("cwd:"))
        .stdout(predicate::str::contains("vsc save 7"));
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
        .stdout(predicate::str::contains("cargo test"));
}

#[test]
fn monorepo_project_filter() {
    let root = fixtures().join("monorepo-mixed");

    // 1. sgl test --project web
    sgl()
        .args(["--dry-run", "test", "--project", "web", "--cwd"])
        .arg(&root)
        .assert()
        .success();

    // 2. sgl --project web test (flag before verb)
    sgl()
        .args(["--dry-run", "--project", "web", "test", "--cwd"])
        .arg(&root)
        .assert()
        .success();

    // 3. Match by relative path: ./web
    sgl()
        .args(["--dry-run", "test", "--project", "./web", "--cwd"])
        .arg(&root)
        .assert()
        .success();

    // 4. Unknown project lists valid project names
    sgl()
        .args([
            "--dry-run",
            "test",
            "--project",
            "nonexistent_proj",
            "--cwd",
        ])
        .arg(&root)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("nonexistent_proj"))
        .stderr(predicate::str::contains("api"))
        .stderr(predicate::str::contains("web"));
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
fn monorepo_flags_serial_changed_and_fail_fast() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();

    // 1. Initialize git repo
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

    let proj_a = root.join("proj_a");
    std::fs::create_dir_all(&proj_a).unwrap();
    std::fs::write(
        proj_a.join("package.json"),
        r#"{"name":"proj_a","scripts":{"test":"node -e \"console.log('out_a')\""}}"#,
    )
    .unwrap();

    let proj_b = root.join("proj_b");
    std::fs::create_dir_all(&proj_b).unwrap();
    std::fs::write(
        proj_b.join("package.json"),
        r#"{"name":"proj_b","scripts":{"test":"node -e \"console.log('out_b')\""}}"#,
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

    // Test 1: Real execution with --serial
    sgl()
        .args(["test", "--serial", "--cwd"])
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("[proj_a] out_a"))
        .stdout(predicate::str::contains("[proj_b] out_b"))
        .stderr(predicate::str::contains("proj_a"))
        .stderr(predicate::str::contains("proj_b"))
        .stderr(predicate::str::contains("success"));

    // Test 2: Modify only proj_a, then run with --changed
    std::fs::write(
        proj_a.join("package.json"),
        r#"{"name":"proj_a","scripts":{"test":"node -e \"console.log('out_a_modified')\""}}"#,
    )
    .unwrap();

    sgl()
        .args(["test", "--changed", "--cwd"])
        .arg(root)
        .assert()
        .success()
        .stdout(predicate::str::contains("[proj_a] out_a_modified"))
        .stdout(predicate::str::contains("out_b").not());

    // Test 3: --fail-fast stops on first failure
    std::fs::write(
        proj_a.join("package.json"),
        r#"{"name":"proj_a","scripts":{"test":"node -e \"process.exit(1)\""}}"#,
    )
    .unwrap();

    sgl()
        .args(["test", "--serial", "--fail-fast", "--cwd"])
        .arg(root)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("proj_a"))
        .stderr(predicate::str::contains("failed"))
        .stderr(predicate::str::contains("skipped"));
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

#[test]
fn doctor_json_never_prints_zero_bytes() {
    let tmp = tempfile::TempDir::new().unwrap();
    let empty_dir = tmp.path();

    let out = sgl()
        .args(["doctor", "--json", "--cwd"])
        .arg(empty_dir)
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "doctor --json printed 0 bytes on empty directory"
    );
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("expected valid JSON from sgl doctor --json");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["tools"].as_array().map(|a| a.len()), Some(0));
}

#[test]
fn doctor_monorepo_discovery_and_project_filter() {
    let root = fixtures().join("monorepo-mixed");

    // Monorepo doctor discovers tools across subprojects and deduplicates
    let out_all = sgl()
        .args(["doctor", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let stdout_all = String::from_utf8_lossy(&out_all.stdout);
    let v_all: serde_json::Value = serde_json::from_str(&stdout_all).unwrap();
    let tools_all: Vec<&str> = v_all["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["tool"].as_str().unwrap())
        .collect();
    assert!(tools_all.contains(&"node"));
    assert!(tools_all.contains(&"npm"));
    assert!(tools_all.contains(&"cargo"));

    // sgl doctor --project web only checks web's tools
    let out_web = sgl()
        .args(["doctor", "--project", "web", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let stdout_web = String::from_utf8_lossy(&out_web.stdout);
    let v_web: serde_json::Value = serde_json::from_str(&stdout_web).unwrap();
    let tools_web: Vec<&str> = v_web["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["tool"].as_str().unwrap())
        .collect();
    assert!(tools_web.contains(&"node"));
    assert!(tools_web.contains(&"npm"));
    assert!(!tools_web.contains(&"cargo"));

    // sgl doctor --project api only checks api's tools
    let out_api = sgl()
        .args(["doctor", "--project", "api", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let stdout_api = String::from_utf8_lossy(&out_api.stdout);
    let v_api: serde_json::Value = serde_json::from_str(&stdout_api).unwrap();
    let tools_api: Vec<&str> = v_api["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["tool"].as_str().unwrap())
        .collect();
    assert!(tools_api.contains(&"cargo"));
    assert!(!tools_api.contains(&"node") && !tools_api.contains(&"npm"));
}

#[test]
fn singularity_toml_shell_command_quoting_with_spaces_and_special_chars() {
    let tmp = tempfile::TempDir::new().unwrap();
    let spaced_cwd = tmp.path().join("dir with spaces").join("my project");
    std::fs::create_dir_all(&spaced_cwd).unwrap();

    let toml_content = r#"[commands]
test = "node -e \"console.log('hi & %')\""
build = "node -e \"console.log(process.argv[1])\" \"double 'quote' & %VAR%\""
"#;
    std::fs::write(spaced_cwd.join("singularity.toml"), toml_content).unwrap();

    let assert_test = sgl()
        .args(["test", "--cwd"])
        .arg(&spaced_cwd)
        .assert()
        .success();

    assert_test.stdout(predicate::str::contains("hi & %"));

    let assert_build = sgl()
        .args(["build", "--cwd"])
        .arg(&spaced_cwd)
        .assert()
        .success();

    assert_build.stdout(predicate::str::contains("double 'quote' & %VAR%"));
}

#[test]
fn quiet_flag_hides_banner_but_prints_payloads() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::write(
        root.join("package.json"),
        r#"{
  "name": "quiet-pkg",
  "version": "1.0.0",
  "scripts": {
    "test": "node -e 'console.log(1)'"
  }
}"#,
    )
    .unwrap();

    // 1. info --quiet must print project payload, but not banner or verb list
    let out_info = sgl()
        .args(["info", "--quiet", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let combined_info = format!(
        "{}{}",
        String::from_utf8_lossy(&out_info.stdout),
        String::from_utf8_lossy(&out_info.stderr)
    );
    assert!(
        combined_info.contains("quiet-pkg"),
        "info --quiet should output project name, got: {combined_info}"
    );
    assert!(
        combined_info.contains("[node]"),
        "info --quiet should output project kind"
    );
    assert!(!combined_info.contains("___(_)"), "banner must be hidden");
    assert!(
        !combined_info.contains("Available commands:"),
        "verb list must be hidden"
    );

    // 2. doctor --quiet must print doctor table
    let out_doc = sgl()
        .args(["doctor", "--quiet", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let combined_doc = format!(
        "{}{}",
        String::from_utf8_lossy(&out_doc.stdout),
        String::from_utf8_lossy(&out_doc.stderr)
    );
    assert!(
        combined_doc.contains("node"),
        "doctor --quiet should print tool name"
    );
    assert!(
        combined_doc.contains("status"),
        "doctor --quiet should print table header"
    );

    // 3. --explain --quiet must print plan details
    let out_exp = sgl()
        .args(["--explain", "test", "--quiet", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let combined_exp = format!(
        "{}{}",
        String::from_utf8_lossy(&out_exp.stdout),
        String::from_utf8_lossy(&out_exp.stderr)
    );
    assert!(
        combined_exp.contains("npm"),
        "--explain --quiet should show program"
    );
    assert!(
        combined_exp.contains("reason:"),
        "--explain --quiet should show reason"
    );

    // 4. dry-run --quiet must print the command line that would run
    let out_dry = sgl()
        .args(["test", "--dry-run", "--quiet", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let combined_dry = format!(
        "{}{}",
        String::from_utf8_lossy(&out_dry.stdout),
        String::from_utf8_lossy(&out_dry.stderr)
    );
    assert!(
        combined_dry.contains("npm"),
        "dry-run --quiet should print the planned command, got: {combined_dry}"
    );

    // 5. error output must still print with --quiet
    let out_err = sgl()
        .args(["lint", "--quiet", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let stderr_err = String::from_utf8_lossy(&out_err.stderr);
    assert!(
        stderr_err.contains("sgl: no 'lint' command for this project"),
        "errors must still print with --quiet, got: {stderr_err}"
    );
}

#[test]
fn schema_version_present_in_all_json_commands() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::write(
        root.join("package.json"),
        r#"{"name": "json-test-pkg", "scripts": {"build": "node -e 'console.log(1)'"}}"#,
    )
    .unwrap();

    // 1. info --json
    let out_info = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out_info.status.success());
    let v_info: serde_json::Value = serde_json::from_slice(&out_info.stdout).unwrap();
    assert_eq!(v_info["schema_version"], 1);

    // 2. doctor --json
    let out_doc = sgl()
        .args(["doctor", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    let v_doc: serde_json::Value = serde_json::from_slice(&out_doc.stdout).unwrap();
    assert_eq!(v_doc["schema_version"], 1);

    // 3. --explain <verb> --json (success)
    let out_exp = sgl()
        .args(["--explain", "build", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out_exp.status.success());
    let v_exp: serde_json::Value = serde_json::from_slice(&out_exp.stdout).unwrap();
    assert_eq!(v_exp["schema_version"], 1);

    // 4. --explain <verb> --json (unavailable command)
    let out_exp_unavail = sgl()
        .args(["--explain", "lint", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert_eq!(out_exp_unavail.status.code(), Some(1));
    let v_exp_unavail: serde_json::Value = serde_json::from_slice(&out_exp_unavail.stdout).unwrap();
    assert_eq!(v_exp_unavail["schema_version"], 1);
    assert!(v_exp_unavail["error"].as_str().is_some());

    // 5. doctor --project <nonexistent> --json
    let out_doc_unavail = sgl()
        .args(["doctor", "--project", "nonexistent", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert_eq!(out_doc_unavail.status.code(), Some(1));
    let v_doc_unavail: serde_json::Value = serde_json::from_slice(&out_doc_unavail.stdout).unwrap();
    assert_eq!(v_doc_unavail["schema_version"], 1);
    assert!(v_doc_unavail["error"].as_str().is_some());
}

#[test]
fn npm_run_and_extra_args_passthrough() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::write(
        root.join("package.json"),
        r#"{"name": "npm-args-pkg", "scripts": {"test": "jest", "build": "esbuild"}}"#,
    )
    .unwrap();
    std::fs::write(root.join("package-lock.json"), "{}").unwrap();

    // 1. sgl run test -- --watch
    let out = sgl()
        .args(["--dry-run", "run", "test", "--cwd"])
        .arg(&root)
        .args(["--", "--watch"])
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("npm run test -- --watch"),
        "expected 'npm run test -- --watch', got: {combined}"
    );

    // 2. sgl test -- --watch
    let out_test = sgl()
        .args(["--dry-run", "test", "--cwd"])
        .arg(&root)
        .args(["--", "--watch"])
        .output()
        .unwrap();
    let combined_test = format!(
        "{}{}",
        String::from_utf8_lossy(&out_test.stdout),
        String::from_utf8_lossy(&out_test.stderr)
    );
    assert!(
        combined_test.contains("npm run test -- --watch"),
        "expected 'npm run test -- --watch', got: {combined_test}"
    );
}

#[test]
fn unavailable_message_format_consistent_everywhere() {
    // 1. dotnet lint (unsupported verb for dotnet)
    let out_dotnet = sgl()
        .args(["lint", "--cwd"])
        .arg(fixtures().join("dotnet-multi"))
        .output()
        .unwrap();
    assert_eq!(out_dotnet.status.code(), Some(1));
    let stderr_dotnet = String::from_utf8_lossy(&out_dotnet.stderr);
    assert!(
        stderr_dotnet.contains("sgl: no 'lint' command for this project (dotnet)"),
        "expected header, got: {stderr_dotnet}"
    );
    assert!(stderr_dotnet.contains("path:"));
    assert!(stderr_dotnet.contains("reason: verb 'lint' is not supported for dotnet projects"));
    assert!(stderr_dotnet.contains("available: setup, build, test, fmt, run"));
    assert!(stderr_dotnet.contains("hint: define 'lint' in singularity.toml under [commands]"));

    // 2. Directory with no project detected
    let tmp = tempfile::TempDir::new().unwrap();
    let empty_dir = tmp.path();
    let out_empty = sgl()
        .args(["lint", "--cwd"])
        .arg(empty_dir)
        .output()
        .unwrap();
    assert_eq!(out_empty.status.code(), Some(1));
    let stderr_empty = String::from_utf8_lossy(&out_empty.stderr);
    assert!(
        stderr_empty.contains("sgl: no 'lint' command for this project (no project detected)"),
        "expected header for empty dir, got: {stderr_empty}"
    );
    assert!(stderr_empty.contains("path:"));
    assert!(stderr_empty.contains("reason: no project detected in this directory"));
    assert!(stderr_empty.contains(
        "available: setup, dev, test, build, lint, fmt, run <script>, doctor, info, init"
    ));
    assert!(stderr_empty.contains("hint: run 'sgl init' to create a singularity.toml"));
}

#[test]
fn justfile_recipe_parsing_and_execution() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    let justfile_content = r#"# A test justfile
build:
    cargo build

test:
    cargo test

custom-task:
    echo "custom task"
"#;
    std::fs::write(root.join("justfile"), justfile_content).unwrap();

    // 1. explain test -> maps to just test
    let out_test = sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out_test.status.success());
    let stdout_test = String::from_utf8_lossy(&out_test.stdout);
    assert!(
        stdout_test.contains("just test"),
        "expected 'just test', got: {stdout_test}"
    );

    // 2. explain build -> maps to just build
    let out_build = sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out_build.status.success());
    let stdout_build = String::from_utf8_lossy(&out_build.stdout);
    assert!(
        stdout_build.contains("just build"),
        "expected 'just build', got: {stdout_build}"
    );

    // 3. run custom-task -> dry-run executes just custom-task
    let out_custom = sgl()
        .args(["--dry-run", "run", "custom-task", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out_custom.status.success());
    let combined_custom = format!(
        "{}{}",
        String::from_utf8_lossy(&out_custom.stdout),
        String::from_utf8_lossy(&out_custom.stderr)
    );
    assert!(
        combined_custom.contains("just custom-task"),
        "expected 'just custom-task', got: {combined_custom}"
    );

    // 4. explain dev -> unavailable with structured message listing available recipes
    let out_dev = sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert_eq!(out_dev.status.code(), Some(1));
    let stderr_dev = String::from_utf8_lossy(&out_dev.stderr);
    assert!(
        stderr_dev.contains("sgl: no 'dev' command for this project (just)"),
        "got: {stderr_dev}"
    );
    assert!(stderr_dev.contains("path:"));
    assert!(stderr_dev.contains("reason: no \"dev\" recipe in justfile"));
    assert!(stderr_dev.contains("available: build, custom-task, test"));
    assert!(stderr_dev.contains(
        "hint: add a \"dev\" recipe to justfile, or define it in singularity.toml under [commands]"
    ));
}
#[test]
fn init_dry_run_does_not_write_files() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();

    sgl()
        .args(["init", "--dry-run", "--cwd"])
        .arg(dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("singularity.toml"))
        .stdout(predicate::str::contains("[commands]"));

    let entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
    assert_eq!(entries.len(), 1, "init --dry-run must not write any files");
}

#[test]
fn init_refuses_to_overwrite_existing_config() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname=\"test\"\n").unwrap();
    std::fs::write(dir.join("singularity.toml"), "[commands]\n").unwrap();

    sgl()
        .args(["init", "--cwd"])
        .arg(dir)
        .assert()
        .success()
        .stderr(predicate::str::contains("already exists"));

    let content = std::fs::read_to_string(dir.join("singularity.toml")).unwrap();
    assert_eq!(content, "[commands]\n");
}

#[test]
fn rust_dev_explain_states_reason() {
    let out = sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cargo"),
        "expected cargo in explain output: {stdout}"
    );
    if which::which("cargo-watch").is_ok() {
        assert!(
            stdout.contains("cargo watch -x run (cargo-watch found)"),
            "expected cargo-watch found in reason: {stdout}"
        );
    } else {
        assert!(
            stdout.contains("cargo run (cargo-watch not on PATH)"),
            "expected not on PATH in reason: {stdout}"
        );
    }
}

#[test]
fn dotnet_single_project_dev() {
    sgl()
        .args(["--explain", "dev", "--cwd"])
        .arg(fixtures().join("dotnet-single"))
        .assert()
        .success()
        .stdout(predicate::str::contains("dotnet run"));
}

#[test]
fn dotnet_multi_project_dev_unavailable() {
    sgl()
        .args(["dev", "--cwd"])
        .arg(fixtures().join("dotnet-multi"))
        .assert()
        .code(1)
        .stderr(predicate::str::contains("sgl: no 'dev' command"));
}

#[test]
fn next_dir_is_skipped_during_discovery() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"nextjs-app","scripts":{"dev":"next dev"}}"#,
    )
    .unwrap();
    let next_dev = root.join(".next").join("dev");
    std::fs::create_dir_all(&next_dev).unwrap();
    std::fs::write(
        next_dev.join("package.json"),
        r#"{"name":"internal","scripts":{"dev":"echo"}}"#,
    )
    .unwrap();

    let out = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("\"internal\""),
        "discover must not enter .next directory, found: {:?}",
        stdout
    );
}

#[test]
fn nested_monorepo_with_workspace_root() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"root","workspaces":["packages/*"]}"#,
    )
    .unwrap();
    let pkg_a = root.join("packages").join("a");
    let pkg_b = root.join("packages").join("b");
    std::fs::create_dir_all(&pkg_a).unwrap();
    std::fs::create_dir_all(&pkg_b).unwrap();
    std::fs::write(
        pkg_a.join("package.json"),
        r#"{"name":"pkg-a","scripts":{"build":"tsc"}}"#,
    )
    .unwrap();
    std::fs::write(
        pkg_b.join("package.json"),
        r#"{"name":"pkg-b","scripts":{"build":"tsc"}}"#,
    )
    .unwrap();

    let out = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = json["projects"].as_array().unwrap();
    assert_eq!(
        arr.len(),
        3,
        "should find root + 2 child packages. found: {:?}",
        arr
    );
    let names: Vec<&str> = arr.iter().map(|o| o["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"root"));
    assert!(names.contains(&"pkg-a"));
    assert!(names.contains(&"pkg-b"));
}

#[test]
fn verbose_output_is_superset_of_normal_output() {
    let normal = sgl()
        .args(["--explain", "test", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    let normal_stdout = String::from_utf8_lossy(&normal.stdout);

    let verbose = sgl()
        .args(["--verbose", "--explain", "test", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    let verbose_stdout = String::from_utf8_lossy(&verbose.stdout);

    for line in normal_stdout.lines() {
        if !line.trim().is_empty() {
            assert!(
                verbose_stdout.contains(line.trim()),
                "verbose missing line from normal: {line:?}\nverbose: {verbose_stdout}"
            );
        }
    }

    assert!(
        verbose_stdout.len() > normal_stdout.len(),
        "verbose output should be longer than normal"
    );
}

#[test]
fn dry_run_prints_command_to_stdout() {
    let out = sgl()
        .args(["--dry-run", "test", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // When stdout is redirected to a pipe (in tests), terminal detection falls back to ASCII "->".
    assert!(
        stdout.contains("-> cargo test"),
        "dry-run command with exact prefix must be on stdout. stdout={stdout:?}"
    );
    assert!(
        stderr.is_empty(),
        "dry-run must not output errors to stderr. stderr={stderr:?}"
    );
}

#[test]
fn empty_dir_message_is_lowercase() {
    let tmp = tempfile::TempDir::new().unwrap();
    sgl()
        .args(["info", "--cwd"])
        .arg(tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::is_match("(?i)no projects").unwrap());

    let out = sgl()
        .args(["info", "--cwd"])
        .arg(tmp.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("No projects"),
        "message must be lowercase: {stderr}"
    );
}

#[test]
fn changed_flag_picks_up_untracked_new_file() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();

    for cmd in &[
        vec!["init"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "t@t.com"],
    ] {
        std::process::Command::new("git")
            .args(cmd)
            .current_dir(root)
            .output()
            .unwrap();
    }

    let proj_a = root.join("proj_a");
    std::fs::create_dir_all(&proj_a).unwrap();
    std::fs::write(
        proj_a.join("package.json"),
        r#"{"name":"proj_a","scripts":{"test":"echo a"}}"#,
    )
    .unwrap();

    let proj_b = root.join("proj_b");
    std::fs::create_dir_all(&proj_b).unwrap();
    std::fs::write(
        proj_b.join("package.json"),
        r#"{"name":"proj_b","scripts":{"test":"echo b"}}"#,
    )
    .unwrap();

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

    std::fs::write(proj_b.join("new_file.js"), "// new").unwrap();

    let out = sgl()
        .args(["--dry-run", "test", "--changed", "--cwd"])
        .arg(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("proj_b"),
        "proj_b with untracked file should be picked up by --changed: {stdout}"
    );
}

#[cfg(windows)]
#[test]
fn singularity_toml_commands_with_special_chars_windows() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    std::fs::write(
        dir.join("singularity.toml"),
        r#"[commands]
test = "cmd /C \"echo hello & echo world\"""#,
    )
    .unwrap();
    sgl().args(["test", "--cwd"]).arg(dir).assert().success();
}

#[test]
fn explain_stdout_and_stderr_routing() {
    // 1. --explain text: stdout has plan, stderr is empty
    let out_text = sgl()
        .args(["--explain", "build", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    assert!(out_text.status.success());
    let stdout_text = String::from_utf8_lossy(&out_text.stdout);
    let stderr_text = String::from_utf8_lossy(&out_text.stderr);
    assert!(
        stdout_text.contains("cargo build"),
        "stdout must contain plan: {stdout_text}"
    );
    assert!(
        stdout_text.contains("reason:"),
        "stdout must contain reason: {stdout_text}"
    );
    assert!(
        stderr_text.is_empty(),
        "stderr must be empty on explain success: {stderr_text}"
    );

    // 2. --explain --json: stdout has JSON, stderr is empty
    let out_json = sgl()
        .args(["--explain", "build", "--json", "--cwd"])
        .arg(fixtures().join("rust"))
        .output()
        .unwrap();
    assert!(out_json.status.success());
    let stdout_json = String::from_utf8_lossy(&out_json.stdout);
    let stderr_json = String::from_utf8_lossy(&out_json.stderr);
    let v: serde_json::Value = serde_json::from_str(&stdout_json).expect("valid json on stdout");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["verb"], "build");
    assert!(
        stderr_json.is_empty(),
        "stderr must be empty on explain json success: {stderr_json}"
    );

    // 3. --explain unavailable verb (error): stdout is empty, stderr has error message
    let out_err = sgl()
        .args(["--explain", "lint", "--cwd"])
        .arg(fixtures().join("compose"))
        .output()
        .unwrap();
    assert_eq!(out_err.status.code(), Some(1));
    let stdout_err = String::from_utf8_lossy(&out_err.stdout);
    let stderr_err = String::from_utf8_lossy(&out_err.stderr);
    assert!(
        stdout_err.is_empty(),
        "stdout must be empty on explain error: {stdout_err}"
    );
    assert!(
        stderr_err.contains("sgl: no 'lint' command for this project (docker)"),
        "stderr must contain error: {stderr_err}"
    );
}

#[test]
fn nextjs_fixture_skip_list_only_lists_root_once() {
    let root = fixtures().join("nextjs");

    // 1. Text output: sgl info lists only root once
    let out = sgl().args(["info", "--cwd"]).arg(&root).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    let lines: Vec<&str> = combined.lines().filter(|l| !l.trim().is_empty()).collect();

    // Must contain root name "nextjs-app"
    assert!(
        lines.iter().any(|l| l.contains("nextjs-app")),
        "must list root project: {combined}"
    );
    // Must NOT contain internal-next-dev, dist-pkg, or target-pkg
    assert!(
        !lines.iter().any(|l| l.contains("internal-next-dev")),
        ".next/dev/package.json must be skipped: {combined}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("dist-pkg")),
        "dist/package.json must be skipped: {combined}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("target-pkg")),
        "target/package.json must be skipped: {combined}"
    );
    // Must list root exactly once (not duplicated)
    let count = lines.iter().filter(|l| l.contains("nextjs-app")).count();
    assert_eq!(
        count, 1,
        "nextjs-app must appear exactly once, got {count}: {lines:?}"
    );

    // 2. JSON output: sgl info --json lists exactly one project
    let out_json = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(&root)
        .output()
        .unwrap();
    assert!(out_json.status.success());
    let stdout_json = String::from_utf8_lossy(&out_json.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout_json).unwrap();
    let projects = v["projects"].as_array().expect("projects array");
    assert_eq!(
        projects.len(),
        1,
        "expected exactly 1 project in JSON, got: {stdout_json}"
    );
    assert_eq!(projects[0]["name"], "nextjs-app");
}

#[test]
fn skip_list_ignores_dist_and_target_package_json() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path();

    std::fs::write(
        root.join("package.json"),
        r#"{"name":"root-pkg","scripts":{"build":"next build"}}"#,
    )
    .unwrap();

    let dist = root.join("dist");
    std::fs::create_dir_all(&dist).unwrap();
    std::fs::write(dist.join("package.json"), r#"{"name":"dist-ignored"}"#).unwrap();

    let target = root.join("target");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("package.json"), r#"{"name":"target-ignored"}"#).unwrap();

    let out = sgl()
        .args(["info", "--json", "--cwd"])
        .arg(root)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let projects = v["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 1, "must only discover root, got: {stdout}");
    assert_eq!(projects[0]["name"], "root-pkg");
}
