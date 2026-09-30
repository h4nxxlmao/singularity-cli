//! Command execution: spawn child processes, stream I/O, forward signals.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::detect::CommandPlan;
use crate::ui::Ui;
use owo_colors::OwoColorize;

/// Execute a command plan, streaming stdout/stderr and passing through stdin.
/// Returns the child's exit code.
pub async fn run(plan: &CommandPlan, ui: &Ui, dry_run: bool) -> Result<i32> {
    let cmd_display = format_cmd(plan);
    ui.print_cmd(&cmd_display);

    if dry_run {
        return Ok(0);
    }

    let resolved_path = match resolve_program(&plan.program, &plan.cwd) {
        Some(p) => p,
        None => {
            if let Some(fallback) = &plan.fallback {
                ui.warn(&format!("  '{}' not found, trying fallback", plan.program));
                return Box::pin(run(fallback, ui, dry_run)).await;
            }
            let hint = get_install_hint(&plan.program);
            ui.error(&format!(
                "sgl: '{}' is required for {} but was not found.",
                plan.program, plan.reason
            ));
            if !hint.is_empty() {
                ui.error(&format!("To install:\n{hint}"));
            }
            return Ok(127);
        }
    };

    let mut cmd = build_command(plan, &resolved_path);
    let mut child = cmd.spawn()?;

    tokio::select! {
        status = child.wait() => {
            let status = status?;
            Ok(status.code().unwrap_or(1))
        }
        _ = tokio::signal::ctrl_c() => {
            let _ = child.kill().await;
            Ok(130)
        }
    }
}

/// Execute multiple plans sequentially (for singularity.toml list commands).
pub async fn run_sequence(plans: &[CommandPlan], ui: &Ui, dry_run: bool) -> Result<i32> {
    for plan in plans {
        let code = run(plan, ui, dry_run).await?;
        if code != 0 {
            return Ok(code);
        }
    }
    Ok(0)
}

/// Resolve a program name to an executable path.
/// Handles relative paths in cwd, Windows .cmd/.bat extensions, and PATH lookup.
pub fn resolve_program(program: &str, cwd: &Path) -> Option<PathBuf> {
    let clean = program.trim_start_matches("./").trim_start_matches(".\\");

    // 1. Direct file in cwd
    let in_cwd = cwd.join(clean);
    if in_cwd.is_file() {
        return Some(in_cwd);
    }

    #[cfg(windows)]
    {
        // Check for gradlew.bat, npm.cmd, etc. in cwd
        for ext in &["bat", "cmd", "exe"] {
            let p = cwd.join(format!("{clean}.{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }

    // 2. PATH resolution via which
    if let Ok(path) = which::which(program) {
        return Some(path);
    }

    #[cfg(windows)]
    {
        // On Windows, try explicit cmd/bat extensions if which bare name did not resolve
        for ext in &["cmd", "bat", "exe", "ps1"] {
            let with_ext = format!("{program}.{ext}");
            if let Ok(path) = which::which(&with_ext) {
                return Some(path);
            }
        }
    }

    None
}

/// Returns install hints for well-known tools.
pub fn get_install_hint(tool: &str) -> String {
    let clean_tool = tool.trim_end_matches(".cmd").trim_end_matches(".exe");
    match clean_tool {
        "node" | "npm" | "npx" => {
            "  brew:   brew install node\n  apt:    sudo apt install nodejs npm\n  winget: winget install OpenJS.NodeJS".to_string()
        }
        "pnpm" => {
            "  brew:   brew install pnpm\n  npm:    npm install -g pnpm\n  winget: winget install pnpm.pnpm".to_string()
        }
        "yarn" => {
            "  brew:   brew install yarn\n  npm:    npm install -g yarn\n  winget: winget install Yarn.Yarn".to_string()
        }
        "bun" => {
            "  brew:   brew install oven-sh/bun/bun\n  curl:   curl -fsSL https://bun.sh/install | bash\n  powershell: irm bun.sh/install.ps1 | iex".to_string()
        }
        "cargo" | "rustc" | "clippy" => {
            "  rustup: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n  winget: winget install Rustlang.Rustup".to_string()
        }
        "cargo-watch" => "  cargo:  cargo install cargo-watch".to_string(),
        "python" | "python3" | "pip" => {
            "  brew:   brew install python\n  apt:    sudo apt install python3 python3-pip\n  winget: winget install Python.Python.3".to_string()
        }
        "uv" => {
            "  brew:   brew install uv\n  curl:   curl -LsSf https://astral.sh/uv/install.sh | sh\n  winget: winget install astral-sh.uv".to_string()
        }
        "poetry" => {
            "  brew:   brew install poetry\n  pip:    pip install poetry\n  winget: winget install tastypymusic.poetry".to_string()
        }
        "pytest" => "  pip:    pip install pytest".to_string(),
        "ruff" => "  brew:   brew install ruff\n  pip:    pip install ruff".to_string(),
        "go" | "gofmt" => {
            "  brew:   brew install go\n  apt:    sudo apt install golang\n  winget: winget install GoLang.Go".to_string()
        }
        "golangci-lint" => {
            "  brew:   brew install golangci-lint\n  go:     go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest".to_string()
        }
        "docker" => {
            "  brew:   brew install --cask docker\n  apt:    sudo apt install docker.io docker-compose-v2\n  winget: winget install Docker.DockerDesktop".to_string()
        }
        "mvn" => {
            "  brew:   brew install maven\n  apt:    sudo apt install maven\n  winget: winget install Apache.Maven".to_string()
        }
        "gradle" => {
            "  brew:   brew install gradle\n  apt:    sudo apt install gradle\n  winget: winget install Gradle.Gradle".to_string()
        }
        "bundle" | "ruby" | "rubocop" | "rspec" => {
            "  brew:   brew install ruby\n  apt:    sudo apt install ruby-full\n  winget: winget install RubyInstallerTeam.RubyWithDevKit.3.2".to_string()
        }
        "dotnet" => {
            "  brew:   brew install --cask dotnet-sdk\n  apt:    sudo apt install dotnet-sdk-8.0\n  winget: winget install Microsoft.DotNet.SDK.8".to_string()
        }
        "make" => {
            "  brew:   brew install make\n  apt:    sudo apt install build-essential\n  winget: winget install GnuWin32.Make".to_string()
        }
        "just" => {
            "  brew:   brew install just\n  cargo:  cargo install just\n  winget: winget install Casey.Just".to_string()
        }
        _ => format!("  brew:   brew install {tool}\n  apt:    sudo apt install {tool}"),
    }
}

pub fn format_cmd(plan: &CommandPlan) -> String {
    if plan.args.is_empty() {
        plan.program.clone()
    } else {
        format!("{} {}", plan.program, plan.args.join(" "))
    }
}

fn build_command(plan: &CommandPlan, program_path: &Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program_path);
    cmd.args(&plan.args);
    cmd.current_dir(&plan.cwd);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    for (k, v) in &plan.env {
        cmd.env(k, v);
    }

    cmd
}

/// Print plan details for --explain mode (does not run).
pub fn explain_plan(plan: &CommandPlan, reason: &str, ui: &Ui) {
    let cmd_str = format_cmd(plan);
    if ui.color {
        eprintln!("  {}", format!("→ {cmd_str}").cyan());
    } else {
        eprintln!("  → {cmd_str}");
    }
    eprintln!("     cwd:    {}", plan.cwd.display());
    if !plan.env.is_empty() {
        let env_display: Vec<String> = plan.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        eprintln!("     env:    {}", env_display.join(", "));
    }
    eprintln!("     reason: {reason}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_program_in_cwd() {
        let tmp = TempDir::new().unwrap();
        let script = tmp.path().join("build.sh");
        fs::write(&script, "#!/bin/sh\n").unwrap();

        let resolved = resolve_program("build.sh", tmp.path()).unwrap();
        assert_eq!(resolved, script);
    }

    #[cfg(windows)]
    #[test]
    fn test_resolve_program_windows_bat_in_cwd() {
        let tmp = TempDir::new().unwrap();
        let gradlew_bat = tmp.path().join("gradlew.bat");
        fs::write(&gradlew_bat, "@echo off\n").unwrap();

        // Resolving "gradlew" should find gradlew.bat
        let resolved = resolve_program("gradlew", tmp.path()).unwrap();
        assert_eq!(resolved, gradlew_bat);

        // Resolving "./gradlew" should also find gradlew.bat
        let resolved2 = resolve_program("./gradlew", tmp.path()).unwrap();
        assert_eq!(resolved2, gradlew_bat);
    }

    #[cfg(windows)]
    #[test]
    fn test_resolve_program_windows_path_cmd() {
        // "cmd" or "powershell" is always on Windows PATH
        let resolved = resolve_program("cmd", Path::new("."));
        assert!(resolved.is_some());
    }

    #[test]
    fn test_install_hints() {
        let hint = get_install_hint("node");
        assert!(hint.contains("brew"));
        assert!(hint.contains("apt"));
        assert!(hint.contains("winget"));

        let hint_rust = get_install_hint("cargo");
        assert!(hint_rust.contains("rustup"));
    }
}
