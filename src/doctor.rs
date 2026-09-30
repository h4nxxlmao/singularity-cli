//! `sgl doctor` — check for required tools and version mismatches.

use std::path::Path;

use owo_colors::OwoColorize;

use crate::detect::{all_detectors, detect_all, ToolRequirement};
use crate::ui::Ui;

#[derive(Debug, serde::Serialize)]
pub struct ToolStatus {
    pub tool: String,
    pub found_version: Option<String>,
    pub required_version: String,
    pub status: &'static str,
    pub fix: Option<String>,
}

/// Run doctor checks for all detected projects in `dir`.
pub fn run(dir: &Path, ui: &Ui, json: bool, project_filter: Option<&str>) -> i32 {
    let subs = crate::monorepo::discover(dir);
    let mut projects =
        if subs.len() > 1 || (!subs.is_empty() && crate::monorepo::is_workspace_root(dir)) {
            subs
        } else {
            let top = detect_all(dir);
            if top.is_empty() {
                subs
            } else {
                top
            }
        };

    if let Some(target) = project_filter {
        let mut valid_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
        valid_names.sort();
        valid_names.dedup();

        projects.retain(|p| crate::monorepo::project_matches_filter(p, target, dir));
        if projects.is_empty() {
            let valid_str = if valid_names.is_empty() {
                "none".to_string()
            } else {
                valid_names.join(", ")
            };
            if json {
                let val = serde_json::json!({
                    "schema_version": 1,
                    "error": format!("sgl: no project named '{target}' found. Valid projects: {valid_str}"),
                    "tools": [],
                });
                println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
            } else {
                ui.error(&format!(
                    "sgl: no project named '{target}' found. Valid projects: {valid_str}"
                ));
            }
            return 1;
        }
    }

    if projects.is_empty() {
        if json {
            let val = serde_json::json!({
                "schema_version": 1,
                "tools": [],
            });
            println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
        } else {
            ui.warn("no projects detected in this directory");
        }
        return 0;
    }

    let detectors = all_detectors();
    let mut rows: Vec<ToolStatus> = Vec::new();
    let mut seen_tools = std::collections::HashSet::new();
    let mut exit_code = 0;

    for project in &projects {
        for detector in &detectors {
            if detector.detect(&project.root).is_some() {
                for req in detector.required_tools(project) {
                    if seen_tools.insert(req.tool.clone()) {
                        let status = check_tool(&req);
                        if status.status == "missing" || status.status == "version mismatch" {
                            exit_code = 1;
                        }
                        rows.push(status);
                    }
                }
                break;
            }
        }
    }

    if json {
        let val = serde_json::json!({
            "schema_version": 1,
            "tools": rows,
        });
        println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
    } else {
        print_table(&rows, ui);
    }

    exit_code
}

fn check_tool(req: &ToolRequirement) -> ToolStatus {
    let resolved = crate::exec::resolve_program(&req.tool, Path::new("."));
    match resolved {
        Some(path) => {
            let version = get_version(&path);
            let status = if req.version_req.is_empty() {
                "ok"
            } else {
                // Simple version check
                match (&version, semver_check(&version, &req.version_req)) {
                    (Some(_), true) => "ok",
                    (Some(_), false) => "version mismatch",
                    (None, _) => "ok", // can't determine version
                }
            };
            ToolStatus {
                tool: req.tool.clone(),
                found_version: version,
                required_version: req.version_req.clone(),
                status,
                fix: None,
            }
        }
        None => {
            let fix = build_fix_hint(&req.install_hint);
            ToolStatus {
                tool: req.tool.clone(),
                found_version: None,
                required_version: req.version_req.clone(),
                status: "missing",
                fix: Some(fix),
            }
        }
    }
}

fn get_version(program_path: &Path) -> Option<String> {
    #[cfg(windows)]
    let output = {
        if crate::exec::is_windows_batch(program_path) {
            let mut cmd = std::process::Command::new("cmd");
            cmd.arg("/C");
            let prog_str = crate::exec::quote_win_arg(&program_path.to_string_lossy());
            let full_cmd = format!("\"{} --version\"", prog_str);
            use std::os::windows::process::CommandExt;
            cmd.raw_arg(&full_cmd);
            cmd.output().ok()?
        } else if crate::exec::is_windows_ps1(program_path) {
            let ps = crate::exec::get_powershell_executable();
            std::process::Command::new(ps)
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
                .arg(program_path)
                .arg("--version")
                .output()
                .ok()?
        } else {
            std::process::Command::new(program_path)
                .arg("--version")
                .output()
                .ok()?
        }
    };
    #[cfg(not(windows))]
    let output = std::process::Command::new(program_path)
        .arg("--version")
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let text = if text.trim().is_empty() {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        text
    };

    // Extract first version-looking token
    for word in text.split_whitespace() {
        let clean = word
            .trim_start_matches(['v', 'V'])
            .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if clean
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return Some(clean.to_string());
        }
    }
    None
}

fn semver_check(found: &Option<String>, req: &str) -> bool {
    if let Some(v) = found {
        if let (Ok(ver), Ok(r)) = (
            semver::Version::parse(v.trim_start_matches('v')),
            semver::VersionReq::parse(req),
        ) {
            return r.matches(&ver);
        }
    }
    true // Give benefit of the doubt if we can't parse
}

fn build_fix_hint(hint: &crate::detect::InstallHint) -> String {
    let mut parts = Vec::new();
    if let Some(brew) = &hint.brew {
        parts.push(format!("brew install {brew}"));
    }
    if let Some(apt) = &hint.apt {
        parts.push(format!("apt install {apt}"));
    }
    if let Some(winget) = &hint.winget {
        parts.push(format!("winget install {winget}"));
    }
    if let Some(url) = &hint.url {
        parts.push(url.clone());
    }
    parts.join(" | ")
}

fn print_table(rows: &[ToolStatus], ui: &Ui) {
    // Column widths
    let w_tool = rows.iter().map(|r| r.tool.len()).max().unwrap_or(4).max(4);
    let w_found = rows
        .iter()
        .map(|r| r.found_version.as_deref().unwrap_or("-").len())
        .max()
        .unwrap_or(5)
        .max(5);
    let w_req = rows
        .iter()
        .map(|r| r.required_version.len())
        .max()
        .unwrap_or(8)
        .max(8);

    let header = format!(
        "{:<w_tool$}  {:<w_found$}  {:<w_req$}  status   fix",
        "tool", "found", "required"
    );
    eprintln!("{header}");
    eprintln!("{}", "-".repeat(header.len() + 20));

    for row in rows {
        let found = row.found_version.as_deref().unwrap_or("-");
        let fix = row.fix.as_deref().unwrap_or("");
        let line = format!(
            "{:<w_tool$}  {:<w_found$}  {:<w_req$}  {:<8} {}",
            row.tool, found, row.required_version, row.status, fix
        );
        if ui.color {
            match row.status {
                "ok" => eprintln!("{}", line.green()),
                "missing" | "version mismatch" => eprintln!("{}", line.red()),
                _ => eprintln!("{line}"),
            }
        } else {
            eprintln!("{line}");
        }
    }
}
