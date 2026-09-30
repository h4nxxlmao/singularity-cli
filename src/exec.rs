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
    if dry_run {
        if ui.color {
            eprintln!("  {}", format!("→ {cmd_display}").cyan());
        } else {
            eprintln!("  → {cmd_display}");
        }
        return Ok(0);
    }

    ui.print_cmd(&cmd_display);

    if let Some(raw_cmd) = &plan.raw_shell {
        let first_token = raw_cmd.split_whitespace().next().unwrap_or("");
        let clean_first = first_token
            .trim_start_matches("./")
            .trim_start_matches(".\\");
        let is_builtin = is_shell_builtin(clean_first);
        let resolved = if is_builtin {
            None
        } else {
            resolve_program(first_token, &plan.cwd)
        };

        if !is_builtin && resolved.is_none() {
            if let Some(fallback) = &plan.fallback {
                ui.warn(&format!("  '{}' not found, trying fallback", first_token));
                return Box::pin(run(fallback, ui, dry_run)).await;
            }
            let (tool_name, os_hint) = get_os_install_hint(first_token);
            if !os_hint.is_empty() {
                ui.error(&format!(
                    "sgl: '{}' not found on PATH (needed for: {}). Install {}: {}",
                    first_token, cmd_display, tool_name, os_hint
                ));
            } else {
                ui.error(&format!(
                    "sgl: '{}' not found on PATH (needed for: {}).",
                    first_token, cmd_display
                ));
            }
            return Ok(127);
        }

        #[cfg(windows)]
        if let Some(ref res_path) = resolved {
            if is_windows_ps1(res_path) {
                let ps = get_powershell_executable();
                let mut c = tokio::process::Command::new(ps);
                c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
                c.arg(res_path);
                let rest = raw_cmd[first_token.len()..].trim();
                if !rest.is_empty() {
                    c.raw_arg(format!(" {}", rest));
                }
                c.current_dir(&plan.cwd);
                c.stdin(Stdio::inherit());
                c.stdout(Stdio::inherit());
                c.stderr(Stdio::inherit());
                for (k, v) in &plan.env {
                    c.env(k, v);
                }
                let mut child = c.spawn()?;
                tokio::select! {
                    status = child.wait() => {
                        let status = status?;
                        return Ok(status.code().unwrap_or(1));
                    }
                    _ = tokio::signal::ctrl_c() => {
                        let _ = child.kill().await;
                        return Ok(130);
                    }
                }
            }
        }

        let mut cmd = build_shell_command(raw_cmd, plan);
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
    } else {
        let resolved_path = match resolve_program(&plan.program, &plan.cwd) {
            Some(p) => p,
            None => {
                if let Some(fallback) = &plan.fallback {
                    ui.warn(&format!("  '{}' not found, trying fallback", plan.program));
                    return Box::pin(run(fallback, ui, dry_run)).await;
                }
                let (tool_name, os_hint) = get_os_install_hint(&plan.program);
                if !os_hint.is_empty() {
                    ui.error(&format!(
                        "sgl: '{}' not found on PATH (needed for: {}). Install {}: {}",
                        plan.program, cmd_display, tool_name, os_hint
                    ));
                } else {
                    ui.error(&format!(
                        "sgl: '{}' not found on PATH (needed for: {}).",
                        plan.program, cmd_display
                    ));
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

#[cfg(windows)]
pub fn refine_resolved_path(path: PathBuf) -> PathBuf {
    if let Some(parent) = path.parent() {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            for ext in &["exe", "cmd", "bat"] {
                let candidate = parent.join(format!("{stem}.{ext}"));
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }
    path
}

/// Resolve a program name to an executable path.
/// Handles relative paths in cwd, Windows .cmd/.bat extensions, and PATH lookup.
pub fn resolve_program(program: &str, cwd: &Path) -> Option<PathBuf> {
    let clean = program.trim_start_matches("./").trim_start_matches(".\\");

    #[cfg(windows)]
    {
        // 1. Direct file or extensions in cwd
        let in_cwd = cwd.join(clean);
        if in_cwd.is_file() {
            return Some(refine_resolved_path(in_cwd));
        }

        // Check for gradlew.bat, npm.cmd, etc. in cwd
        for ext in &["exe", "cmd", "bat", "ps1"] {
            let p = cwd.join(format!("{clean}.{ext}"));
            if p.is_file() {
                return Some(refine_resolved_path(p));
            }
        }
    }

    #[cfg(not(windows))]
    {
        let in_cwd = cwd.join(clean);
        if in_cwd.is_file() {
            return Some(in_cwd);
        }
    }

    // 2. PATH resolution via which
    if let Ok(path) = which::which(program) {
        #[cfg(windows)]
        {
            return Some(refine_resolved_path(path));
        }
        #[cfg(not(windows))]
        {
            return Some(path);
        }
    }

    #[cfg(windows)]
    {
        // On Windows, try explicit extensions if which bare name did not resolve
        for ext in &["exe", "cmd", "bat", "ps1"] {
            let with_ext = format!("{program}.{ext}");
            if let Ok(path) = which::which(&with_ext) {
                return Some(refine_resolved_path(path));
            }
        }
    }

    None
}

/// Returns (Tool Name, OS-specific install command)
pub fn get_os_install_hint(tool: &str) -> (&'static str, String) {
    let clean = tool
        .trim_end_matches(".cmd")
        .trim_end_matches(".bat")
        .trim_end_matches(".exe");

    #[cfg(windows)]
    match clean {
        "node" | "npm" | "npx" => ("Node.js", "winget install OpenJS.NodeJS".to_string()),
        "pnpm" => ("pnpm", "winget install pnpm.pnpm".to_string()),
        "yarn" => ("Yarn", "winget install Yarn.Yarn".to_string()),
        "bun" => (
            "Bun",
            "powershell -c \"irm bun.sh/install.ps1 | iex\"".to_string(),
        ),
        "cargo" | "rustc" | "clippy" => ("Rust", "winget install Rustlang.Rustup".to_string()),
        "cargo-watch" => ("cargo-watch", "cargo install cargo-watch".to_string()),
        "python" | "python3" | "pip" => ("Python", "winget install Python.Python.3".to_string()),
        "uv" => ("uv", "winget install astral-sh.uv".to_string()),
        "poetry" => ("Poetry", "winget install tastypymusic.poetry".to_string()),
        "pytest" => ("pytest", "pip install pytest".to_string()),
        "ruff" => ("ruff", "pip install ruff".to_string()),
        "go" | "gofmt" => ("Go", "winget install GoLang.Go".to_string()),
        "golangci-lint" => (
            "golangci-lint",
            "go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest".to_string(),
        ),
        "docker" => ("Docker", "winget install Docker.DockerDesktop".to_string()),
        "mvn" => ("Maven", "winget install Apache.Maven".to_string()),
        "gradle" => ("Gradle", "winget install Gradle.Gradle".to_string()),
        "ruby" | "bundle" | "rubocop" | "rspec" => (
            "Ruby",
            "winget install RubyInstallerTeam.RubyWithDevKit.3.2".to_string(),
        ),
        "dotnet" => (
            ".NET SDK",
            "winget install Microsoft.DotNet.SDK.8".to_string(),
        ),
        "make" => ("Make", "winget install GnuWin32.Make".to_string()),
        "just" => ("just", "winget install Casey.Just".to_string()),
        _ => ("tool", format!("winget install {clean}")),
    }

    #[cfg(target_os = "macos")]
    match clean {
        "node" | "npm" | "npx" => ("Node.js", "brew install node".to_string()),
        "pnpm" => ("pnpm", "brew install pnpm".to_string()),
        "yarn" => ("Yarn", "brew install yarn".to_string()),
        "bun" => ("Bun", "brew install oven-sh/bun/bun".to_string()),
        "cargo" | "rustc" | "clippy" => (
            "Rust",
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string(),
        ),
        "cargo-watch" => ("cargo-watch", "cargo install cargo-watch".to_string()),
        "python" | "python3" | "pip" => ("Python", "brew install python".to_string()),
        "uv" => ("uv", "brew install uv".to_string()),
        "poetry" => ("Poetry", "brew install poetry".to_string()),
        "pytest" => ("pytest", "pip install pytest".to_string()),
        "ruff" => ("ruff", "brew install ruff".to_string()),
        "go" | "gofmt" => ("Go", "brew install go".to_string()),
        "golangci-lint" => ("golangci-lint", "brew install golangci-lint".to_string()),
        "docker" => ("Docker", "brew install --cask docker".to_string()),
        "mvn" => ("Maven", "brew install maven".to_string()),
        "gradle" => ("Gradle", "brew install gradle".to_string()),
        "ruby" | "bundle" | "rubocop" | "rspec" => ("Ruby", "brew install ruby".to_string()),
        "dotnet" => (".NET SDK", "brew install --cask dotnet-sdk".to_string()),
        "make" => ("Make", "brew install make".to_string()),
        "just" => ("just", "brew install just".to_string()),
        _ => ("tool", format!("brew install {clean}")),
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    match clean {
        "node" | "npm" | "npx" => ("Node.js", "sudo apt install nodejs npm".to_string()),
        "pnpm" => ("pnpm", "npm install -g pnpm".to_string()),
        "yarn" => ("Yarn", "npm install -g yarn".to_string()),
        "bun" => (
            "Bun",
            "curl -fsSL https://bun.sh/install | bash".to_string(),
        ),
        "cargo" | "rustc" | "clippy" => (
            "Rust",
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string(),
        ),
        "cargo-watch" => ("cargo-watch", "cargo install cargo-watch".to_string()),
        "python" | "python3" | "pip" => {
            ("Python", "sudo apt install python3 python3-pip".to_string())
        }
        "uv" => (
            "uv",
            "curl -LsSf https://astral.sh/uv/install.sh | sh".to_string(),
        ),
        "poetry" => ("Poetry", "pip install poetry".to_string()),
        "pytest" => ("pytest", "pip install pytest".to_string()),
        "ruff" => ("ruff", "pip install ruff".to_string()),
        "go" | "gofmt" => ("Go", "sudo apt install golang".to_string()),
        "golangci-lint" => (
            "golangci-lint",
            "go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest".to_string(),
        ),
        "docker" => (
            "Docker",
            "sudo apt install docker.io docker-compose-v2".to_string(),
        ),
        "mvn" => ("Maven", "sudo apt install maven".to_string()),
        "gradle" => ("Gradle", "sudo apt install gradle".to_string()),
        "ruby" | "bundle" | "rubocop" | "rspec" => {
            ("Ruby", "sudo apt install ruby-full".to_string())
        }
        "dotnet" => (".NET SDK", "sudo apt install dotnet-sdk-8.0".to_string()),
        "make" => ("Make", "sudo apt install build-essential".to_string()),
        "just" => ("just", "cargo install just".to_string()),
        _ => ("tool", format!("sudo apt install {clean}")),
    }
}

/// Returns install hints for well-known tools across platforms (legacy helper).
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
    if let Some(raw) = &plan.raw_shell {
        return raw.clone();
    }
    if plan.args.is_empty() {
        plan.program.clone()
    } else {
        format!("{} {}", plan.program, plan.args.join(" "))
    }
}

pub fn is_windows_batch(path: &Path) -> bool {
    #[cfg(windows)]
    {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            return ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat");
        }
    }
    let _ = path;
    false
}

pub fn is_windows_ps1(path: &Path) -> bool {
    #[cfg(windows)]
    {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            return ext.eq_ignore_ascii_case("ps1");
        }
    }
    let _ = path;
    false
}

#[cfg(windows)]
pub fn get_powershell_executable() -> &'static str {
    if which::which("powershell").is_ok() {
        "powershell"
    } else if which::which("pwsh").is_ok() {
        "pwsh"
    } else {
        "powershell"
    }
}

#[cfg(windows)]
pub fn quote_win_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    if !arg.contains(' ') && !arg.contains('\t') && !arg.contains('"') {
        return arg.to_string();
    }
    let mut quoted = String::from("\"");
    for ch in arg.chars() {
        if ch == '"' {
            quoted.push_str("\\\"");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('"');
    quoted
}

pub fn build_command(plan: &CommandPlan, program_path: &Path) -> tokio::process::Command {
    let mut cmd = if is_windows_batch(program_path) {
        #[cfg(windows)]
        {
            let mut c = tokio::process::Command::new("cmd");
            c.arg("/C");
            let prog_str = quote_win_arg(&program_path.to_string_lossy());
            let mut parts = vec![prog_str];
            for a in &plan.args {
                parts.push(quote_win_arg(a));
            }
            let full_cmd = format!("\"{}\"", parts.join(" "));
            c.raw_arg(&full_cmd);
            c
        }
        #[cfg(not(windows))]
        {
            let mut c = tokio::process::Command::new(program_path);
            c.args(&plan.args);
            c
        }
    } else if is_windows_ps1(program_path) {
        #[cfg(windows)]
        {
            let ps = get_powershell_executable();
            let mut c = tokio::process::Command::new(ps);
            c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]);
            c.arg(program_path);
            c.args(&plan.args);
            c
        }
        #[cfg(not(windows))]
        {
            let mut c = tokio::process::Command::new(program_path);
            c.args(&plan.args);
            c
        }
    } else {
        let mut c = tokio::process::Command::new(program_path);
        c.args(&plan.args);
        c
    };

    cmd.current_dir(&plan.cwd);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    for (k, v) in &plan.env {
        cmd.env(k, v);
    }

    cmd
}

pub fn is_shell_builtin(cmd: &str) -> bool {
    let lower = cmd.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "echo"
            | "cd"
            | "dir"
            | "copy"
            | "del"
            | "erase"
            | "mkdir"
            | "md"
            | "rmdir"
            | "rd"
            | "type"
            | "set"
            | "cls"
            | "exit"
            | "rem"
            | "ren"
            | "rename"
            | "move"
            | "start"
            | "call"
            | "pwd"
            | "export"
            | "source"
            | "eval"
            | "exec"
            | "read"
            | "true"
            | "false"
            | "test"
    )
}

pub fn build_shell_command(raw_cmd: &str, plan: &CommandPlan) -> tokio::process::Command {
    #[cfg(windows)]
    let raw_to_run = if let Some(stripped) = raw_cmd.strip_prefix("./") {
        format!(".\\{stripped}")
    } else {
        raw_cmd.to_string()
    };
    #[cfg(not(windows))]
    let raw_to_run = raw_cmd.to_string();

    #[cfg(windows)]
    let mut cmd = {
        let mut c = tokio::process::Command::new("cmd");
        c.arg("/C");
        c.raw_arg(format!("\"{}\"", raw_to_run));
        c
    };

    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = tokio::process::Command::new("sh");
        c.arg("-c");
        c.arg(raw_to_run);
        c
    };

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

    #[cfg(windows)]
    #[tokio::test]
    async fn test_cmd_shim_execution() {
        let tmp = tempfile::tempdir().unwrap();
        let spaced_dir = tmp.path().join("dir with spaces");
        std::fs::create_dir_all(&spaced_dir).unwrap();
        let cmd_file = spaced_dir.join("test shim.cmd");
        std::fs::write(
            &cmd_file,
            "@echo off\r\nif \"%~1\"==\"arg with spaces\" exit 0\r\nexit 42\r\n",
        )
        .unwrap();

        // Test raw_arg
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.arg("/C");
        let cmd_str = format!("\"\"{}\" \"arg with spaces\"\"", cmd_file.display());
        cmd.raw_arg(&cmd_str);
        cmd.current_dir(&spaced_dir);

        let status = cmd.status().await.unwrap();
        assert_eq!(status.code(), Some(0));
    }
}
