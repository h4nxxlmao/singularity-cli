//! `sgl doctor` — check for required tools and version mismatches.

use std::path::Path;

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
pub fn run(dir: &Path, ui: &Ui, json: bool) -> i32 {
    let projects = detect_all(dir);
    let detectors = all_detectors();

    let mut rows: Vec<ToolStatus> = Vec::new();
    let mut exit_code = 0;

    for project in &projects {
        // Find the matching detector
        for detector in &detectors {
            if detector.detect(&project.root).is_some() {
                for req in detector.required_tools(project) {
                    let status = check_tool(&req);
                    if status.status == "missing" || status.status == "version mismatch" {
                        exit_code = 1;
                    }
                    rows.push(status);
                }
                break;
            }
        }
    }

    if projects.is_empty() {
        if !json {
            ui.warn("no projects detected in this directory");
        }
        return 0;
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_default()
        );
    } else {
        print_table(&rows, ui);
    }

    exit_code
}

fn check_tool(req: &ToolRequirement) -> ToolStatus {
    match which::which(&req.tool) {
        Ok(_path) => {
            let version = get_version(&req.tool);
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
        Err(_) => {
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

fn get_version(tool: &str) -> Option<String> {
    let output = std::process::Command::new(tool)
        .arg("--version")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    // Extract first version-looking token
    for word in text.split_whitespace() {
        if word
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return Some(
                word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '.')
                    .to_string(),
            );
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
        match row.status {
            "ok" => ui.success(&line),
            "missing" | "version mismatch" => ui.error(&line),
            _ => eprintln!("{line}"),
        }
    }
}
