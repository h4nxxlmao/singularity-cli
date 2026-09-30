//! Monorepo discovery and parallel execution engine.

use anyhow::Result;
use ignore::WalkBuilder;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::config::Config;
use crate::detect::{all_detectors, CommandPlan, Project, Verb};
use crate::ui::Ui;
use owo_colors::OwoColorize;

/// Options for monorepo execution.
#[derive(Debug, Clone)]
pub struct MonorepoOptions {
    pub verb: Verb,
    pub extra_args: Vec<String>,
    pub project_filter: Option<String>,
    pub changed: bool,
    pub serial: bool,
    pub fail_fast: bool,
    pub dry_run: bool,
}

/// The outcome of running a verb in a project.
#[derive(Debug, Clone)]
pub struct ProjectResult {
    pub project: String,
    pub verb: String,
    pub status: ProjectStatus,
    pub duration: std::time::Duration,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStatus {
    Success,
    Failed,
    Skipped,
}

/// Walk `root` up to `max_depth` levels (default 4), respecting .gitignore,
/// and return all detected sub-projects.
pub fn discover(root: &Path) -> Vec<Project> {
    discover_with_depth(root, 4)
}

/// Walk `root` up to `max_depth` levels, respecting .gitignore,
/// and return all detected sub-projects.
pub fn discover_with_depth(root: &Path, max_depth: usize) -> Vec<Project> {
    let mut projects: Vec<Project> = Vec::new();
    let skip_dirs = [
        "node_modules",
        "target",
        ".venv",
        "venv",
        "__pycache__",
        "dist",
        "build",
        "out",
        ".next",
        ".nuxt",
        ".svelte-kit",
        ".turbo",
        ".cache",
        ".gradle",
        ".idea",
        ".vscode",
        "bin",
        "obj",
        "vendor",
        ".git",
    ];

    let walker = WalkBuilder::new(root)
        .max_depth(Some(max_depth))
        .hidden(false)
        .git_ignore(true)
        .require_git(false)
        .filter_entry(move |entry| {
            let name = entry.file_name().to_string_lossy();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                !skip_dirs.contains(&name.as_ref())
            } else {
                true
            }
        })
        .build();

    let detectors = all_detectors();

    for entry in walker.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let dir = entry.path();
        // Skip root itself (sub-projects only)
        if entry.depth() == 0 || dir == root {
            continue;
        }

        for detector in &detectors {
            if let Some(project) = detector.detect(dir) {
                // Avoid duplicates (same root)
                if !projects.iter().any(|p| p.root == project.root) {
                    projects.push(project);
                }
                break;
            }
        }
    }

    projects
}

/// Returns true if `dir` contains more than one detectable project, or is a workspace root.
pub fn is_monorepo(root: &Path) -> bool {
    let subs = discover(root);
    subs.len() > 1 || (!subs.is_empty() && is_workspace_root(root))
}

pub fn is_workspace_root(root: &Path) -> bool {
    // Check package.json workspaces
    if let Ok(c) = std::fs::read_to_string(root.join("package.json")) {
        if c.contains("\"workspaces\"") {
            return true;
        }
    }
    // Check pnpm-workspace.yaml
    if root.join("pnpm-workspace.yaml").exists() {
        return true;
    }
    // Check Cargo.toml [workspace]
    if let Ok(c) = std::fs::read_to_string(root.join("Cargo.toml")) {
        if c.contains("[workspace]") {
            return true;
        }
    }
    false
}

pub fn project_matches_filter(p: &Project, target: &str, root: &Path) -> bool {
    if p.name == target || p.name.eq_ignore_ascii_case(target) {
        return true;
    }

    let target_clean = target
        .trim_start_matches("./")
        .trim_start_matches(".\\")
        .trim_end_matches('/')
        .trim_end_matches('\\');

    if let Ok(rel) = p.root.strip_prefix(root) {
        let rel_norm = rel.to_string_lossy().replace('\\', "/");
        let target_norm = target_clean.replace('\\', "/");
        if rel_norm == target_norm || rel_norm.eq_ignore_ascii_case(&target_norm) {
            return true;
        }
    }

    let target_path = root.join(target_clean);
    let p_norm = crate::normalize_path(&p.root);
    let t_norm = crate::normalize_path(&target_path);
    if p_norm == t_norm {
        return true;
    }

    false
}

/// Run a verb across all matching projects in a monorepo.
pub async fn run_monorepo(
    root: &Path,
    opts: MonorepoOptions,
    ui: &Ui,
    cfg: Option<&Config>,
) -> Result<i32> {
    let mut projects = discover(root);

    if projects.is_empty() {
        ui.warn("no projects detected in monorepo");
        return Ok(0);
    }

    // 1. Filter by --project <NAME>
    if let Some(target) = &opts.project_filter {
        let mut valid_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
        valid_names.sort();
        valid_names.dedup();

        projects.retain(|p| project_matches_filter(p, target, root));
        if projects.is_empty() {
            let valid_str = if valid_names.is_empty() {
                "none".to_string()
            } else {
                valid_names.join(", ")
            };
            ui.error(&format!(
                "sgl: no project named '{target}' found. Valid projects: {valid_str}"
            ));
            return Ok(1);
        }
    }

    // 2. Filter by --changed
    if opts.changed {
        let repo_root = crate::git::repo_root(root).unwrap_or_else(|| root.to_path_buf());
        let changed_files = crate::git::changed_files(&repo_root, None);
        projects = crate::git::filter_changed_projects(projects, &changed_files, &repo_root);
        if projects.is_empty() {
            ui.warn("no changed projects found");
            return Ok(0);
        }
    }

    // 3. Special handling for Verb::Dev: run all projects concurrently until Ctrl+C
    if opts.verb == Verb::Dev {
        return run_dev_monorepo(projects, &opts, ui, cfg).await;
    }

    // 4. Standard execution with concurrency limit and summary table
    let concurrency = if opts.serial {
        1
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    };

    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
    let abort_flag = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();

    let verb_name = match &opts.verb {
        Verb::Run(s) => format!("run {s}"),
        v => v.name().to_string(),
    };

    for project in projects {
        let sem = Arc::clone(&semaphore);
        let abort = Arc::clone(&abort_flag);
        let verb = opts.verb.clone();
        let extra_args = opts.extra_args.clone();
        let dry_run = opts.dry_run;
        let quiet = ui.quiet;
        let unicode = ui.unicode;
        let fail_fast = opts.fail_fast;
        let verb_display = verb_name.clone();

        // Resolve plan for this project
        let resolutions = crate::resolve::resolve(&project.root, &verb, None, cfg);

        handles.push(tokio::spawn(async move {
            let start = Instant::now();

            if abort.load(Ordering::SeqCst) {
                return ProjectResult {
                    project: project.name,
                    verb: verb_display,
                    status: ProjectStatus::Skipped,
                    duration: start.elapsed(),
                    exit_code: 0,
                };
            }

            let resolutions = match resolutions {
                Ok(r) => r,
                Err(_) => {
                    return ProjectResult {
                        project: project.name,
                        verb: verb_display,
                        status: ProjectStatus::Skipped,
                        duration: start.elapsed(),
                        exit_code: 0,
                    };
                }
            };

            let _permit = sem.acquire().await;
            if abort.load(Ordering::SeqCst) {
                return ProjectResult {
                    project: project.name,
                    verb: verb_display,
                    status: ProjectStatus::Skipped,
                    duration: start.elapsed(),
                    exit_code: 0,
                };
            }

            let mut code = 0;
            for mut res in resolutions {
                if let Some(last) = res.plans.last_mut() {
                    last.append_extra_args(&extra_args);
                }

                for plan in res.plans {
                    let c =
                        run_prefixed_command(&plan, &project.name, dry_run, quiet, unicode).await;
                    match c {
                        Ok(exit) if exit != 0 => {
                            code = exit;
                            break;
                        }
                        Err(_) => {
                            code = 1;
                            break;
                        }
                        _ => {}
                    }
                }

                if code != 0 {
                    break;
                }
            }

            let duration = start.elapsed();
            let status = if code == 0 {
                ProjectStatus::Success
            } else {
                if fail_fast {
                    abort.store(true, Ordering::SeqCst);
                }
                ProjectStatus::Failed
            };

            ProjectResult {
                project: project.name,
                verb: verb_display,
                status,
                duration,
                exit_code: code,
            }
        }));
    }

    let mut results = Vec::new();
    let mut exit_code = 0;

    for handle in handles {
        if let Ok(res) = handle.await {
            if res.exit_code != 0 && exit_code == 0 {
                exit_code = res.exit_code;
            }
            results.push(res);
        }
    }

    print_summary_table(&results, ui);

    Ok(exit_code)
}

/// Run all dev projects simultaneously until Ctrl+C is pressed.
async fn run_dev_monorepo(
    projects: Vec<Project>,
    opts: &MonorepoOptions,
    ui: &Ui,
    cfg: Option<&Config>,
) -> Result<i32> {
    let mut children = Vec::new();

    for project in projects {
        if let Ok(resolutions) = crate::resolve::resolve(&project.root, &opts.verb, None, cfg) {
            for res in resolutions {
                for plan in res.plans {
                    let cmd_str = format!("[{}] {}", project.name, crate::exec::format_cmd(&plan));
                    if opts.dry_run {
                        let arrow = ui.arrow();
                        if ui.color {
                            println!("  {}", format!("{arrow} {cmd_str}").cyan());
                        } else {
                            println!("  {arrow} {cmd_str}");
                        }
                        continue;
                    }
                    ui.print_cmd(&cmd_str);
                    let mut cmd = if let Some(raw) = &plan.raw_shell {
                        crate::exec::build_shell_command(raw, &plan)
                    } else if let Some(resolved) =
                        crate::exec::resolve_program(&plan.program, &plan.cwd)
                    {
                        crate::exec::build_command(&plan, &resolved)
                    } else {
                        continue;
                    };
                    cmd.stdout(Stdio::piped());
                    cmd.stderr(Stdio::piped());

                    if let Ok(mut child) = cmd.spawn() {
                        let stdout = child.stdout.take();
                        let stderr = child.stderr.take();
                        let prefix = project.name.clone();

                        tokio::spawn(async move {
                            if let Some(out) = stdout {
                                use tokio::io::{AsyncBufReadExt, BufReader};
                                let mut reader = BufReader::new(out).lines();
                                while let Ok(Some(line)) = reader.next_line().await {
                                    println!("[{prefix}] {line}");
                                }
                            }
                        });

                        let prefix_err = project.name.clone();
                        tokio::spawn(async move {
                            if let Some(err) = stderr {
                                use tokio::io::{AsyncBufReadExt, BufReader};
                                let mut reader = BufReader::new(err).lines();
                                while let Ok(Some(line)) = reader.next_line().await {
                                    eprintln!("[{prefix_err}] {line}");
                                }
                            }
                        });

                        children.push(child);
                    }
                }
            }
        }
    }

    if opts.dry_run {
        return Ok(0);
    }

    // Wait until Ctrl+C
    let _ = tokio::signal::ctrl_c().await;
    ui.warn("sgl: received interrupt, terminating dev processes...");

    for mut child in children {
        let _ = child.kill().await;
    }

    Ok(0)
}

/// Run a command plan, streaming stdout/stderr with a `[project]` prefix.
async fn run_prefixed_command(
    plan: &CommandPlan,
    project_name: &str,
    dry_run: bool,
    quiet: bool,
    unicode: bool,
) -> Result<i32> {
    let arrow = if unicode { "→" } else { "->" };
    let cmd_str = crate::exec::format_cmd(plan);
    if dry_run {
        println!("  {arrow} [{project_name}] {cmd_str}");
        return Ok(0);
    }

    if !quiet {
        println!("  {arrow} [{project_name}] {cmd_str}");
    }

    let mut cmd = if let Some(raw) = &plan.raw_shell {
        crate::exec::build_shell_command(raw, plan)
    } else {
        let program_path = match crate::exec::resolve_program(&plan.program, &plan.cwd) {
            Some(p) => p,
            None => {
                eprintln!(
                    "[{project_name}] sgl: '{}' not found for {}",
                    plan.program, plan.reason
                );
                return Ok(127);
            }
        };
        crate::exec::build_command(plan, &program_path)
    };
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let prefix_out = project_name.to_string();
    let stdout_task = tokio::spawn(async move {
        if let Some(out) = stdout {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(out).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("[{prefix_out}] {line}");
            }
        }
    });

    let prefix_err = project_name.to_string();
    let stderr_task = tokio::spawn(async move {
        if let Some(err) = stderr {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut reader = BufReader::new(err).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                eprintln!("[{prefix_err}] {line}");
            }
        }
    });

    let status = child.wait().await?;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    Ok(status.code().unwrap_or(1))
}

/// Print the summary table after monorepo execution.
fn print_summary_table(results: &[ProjectResult], ui: &Ui) {
    if results.is_empty() {
        return;
    }

    let w_proj = results
        .iter()
        .map(|r| r.project.len())
        .max()
        .unwrap_or(7)
        .max(7);
    let w_verb = results
        .iter()
        .map(|r| r.verb.len())
        .max()
        .unwrap_or(4)
        .max(4);

    let header = format!(
        "{:<w_proj$}  {:<w_verb$}  status   duration",
        "project", "verb"
    );
    eprintln!("\n{header}");
    eprintln!("{}", "-".repeat(header.len() + 8));

    for r in results {
        let dur = format!("{:.2}s", r.duration.as_secs_f64());
        let status_str = match r.status {
            ProjectStatus::Success => "success",
            ProjectStatus::Failed => "failed",
            ProjectStatus::Skipped => "skipped",
        };
        let line = format!(
            "{:<w_proj$}  {:<w_verb$}  {:<7}  {}",
            r.project, r.verb, status_str, dur
        );
        if ui.color {
            match r.status {
                ProjectStatus::Success => eprintln!("{}", line.green()),
                ProjectStatus::Failed => eprintln!("{}", line.red()),
                ProjectStatus::Skipped => eprintln!("{}", line.yellow()),
            }
        } else {
            eprintln!("{line}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discovers_sub_projects() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create two sub-projects
        let web = root.join("web");
        fs::create_dir_all(&web).unwrap();
        fs::write(
            web.join("package.json"),
            r#"{"name":"web","scripts":{"build":"vite build"}}"#,
        )
        .unwrap();

        let api = root.join("api");
        fs::create_dir_all(&api).unwrap();
        fs::write(
            api.join("Cargo.toml"),
            "[package]\nname=\"api\"\nversion=\"0.1.0\"",
        )
        .unwrap();

        let found = discover(root);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_depth_limit() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Level 1: a/b/c/d/e (depth 5)
        let deep = root.join("a").join("b").join("c").join("d").join("e");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("Cargo.toml"), "[package]\nname=\"deep\"").unwrap();

        // Max depth 4 shouldn't find depth 5
        let found = discover_with_depth(root, 4);
        assert!(
            found.is_empty(),
            "expected deep project to be excluded by depth 4 limit"
        );
    }

    #[test]
    fn skips_node_modules() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let nm = root.join("node_modules").join("some-pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("package.json"), r#"{"name":"some-pkg"}"#).unwrap();

        let found = discover(root);
        assert!(found.is_empty());
    }
}
