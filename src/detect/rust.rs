//! Rust project detector.

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct RustDetector;

impl Detector for RustDetector {
    fn id(&self) -> &'static str {
        "rust"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        let cargo_path = dir.join("Cargo.toml");
        if !cargo_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&cargo_path).unwrap_or_default();
        let name = extract_cargo_name(&content).unwrap_or_else(|| dir_name(dir).to_string());

        Some(Project {
            name,
            root: dir.to_path_buf(),
            kind: ProjectKind::Rust,
            package_manager: Some("cargo".to_string()),
            scripts: HashMap::new(),
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let cwd = project.root.clone();

        let is_workspace = std::fs::read_to_string(cwd.join("Cargo.toml"))
            .map(|c| c.contains("[workspace]"))
            .unwrap_or(false);

        let plan = match verb {
            Verb::Setup => {
                CommandPlan::simple("cargo", ["fetch"], cwd, "cargo fetch: Cargo.toml found")
            }
            Verb::Dev => {
                if which::which("cargo-watch").is_ok() {
                    CommandPlan::simple(
                        "cargo",
                        ["watch", "-x", "run"],
                        cwd,
                        "cargo watch -x run (cargo-watch found)",
                    )
                } else {
                    CommandPlan::simple(
                        "cargo",
                        ["run"],
                        cwd,
                        "cargo run (cargo-watch not on PATH)",
                    )
                }
            }
            Verb::Test => {
                CommandPlan::simple("cargo", ["test"], cwd, "cargo test: Cargo.toml found")
            }
            Verb::Build => {
                CommandPlan::simple("cargo", ["build"], cwd, "cargo build: Cargo.toml found")
            }
            Verb::Lint => {
                let args = if is_workspace {
                    vec!["clippy", "--workspace", "--", "-D", "warnings"]
                } else {
                    vec!["clippy", "--", "-D", "warnings"]
                };
                CommandPlan::simple("cargo", args, cwd, "cargo clippy: Cargo.toml found")
            }
            Verb::Fmt => {
                let args = if is_workspace {
                    vec!["fmt", "--all"]
                } else {
                    vec!["fmt"]
                };
                CommandPlan::simple("cargo", args, cwd, "cargo fmt: Cargo.toml found")
            }
            Verb::Run(script) => CommandPlan::simple(
                "cargo",
                ["run", "--bin", script.as_str()],
                cwd,
                "cargo run --bin",
            ),
        };
        Some(plan)
    }

    fn required_tools(&self, _project: &Project) -> Vec<ToolRequirement> {
        vec![ToolRequirement {
            tool: "cargo".to_string(),
            version_req: String::new(),
            install_hint: InstallHint {
                brew: Some("rustup".to_string()),
                apt: None,
                winget: Some("Rustlang.Rustup".to_string()),
                url: Some("https://rustup.rs".to_string()),
            },
        }]
    }
}

fn extract_cargo_name(toml_content: &str) -> Option<String> {
    // Simple line-based extraction to avoid full TOML parse just for a name.
    for line in toml_content.lines() {
        let line = line.trim();
        if line.starts_with("name") && line.contains('=') {
            let val = line.split_once('=')?.1.trim();
            return Some(val.trim_matches('"').trim_matches('\'').to_string());
        }
    }
    None
}

fn dir_name(dir: &Path) -> &str {
    dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn detects_rust() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"myapp\"\nversion = \"0.1.0\"",
        )
        .unwrap();

        let d = RustDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::Rust);
        assert_eq!(p.name, "myapp");
    }

    #[test]
    fn plan_test() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();

        let d = RustDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Test).unwrap();
        assert_eq!(plan.program, "cargo");
        assert!(plan.args.contains(&"test".to_string()));
    }

    #[test]
    fn plan_lint() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();

        let d = RustDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Lint).unwrap();
        assert!(plan.args.contains(&"clippy".to_string()));
        assert!(plan.args.contains(&"-D".to_string()));
    }

    #[test]
    fn no_cargo_toml_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = RustDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
#[test]
fn dev_uses_cargo_run_when_cargo_watch_absent() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    std::fs::write(dir.join("Cargo.toml"), "[package]\nname=\"app\"").unwrap();

    let d = RustDetector;
    let p = <RustDetector as super::Detector>::detect(&d, dir).unwrap();

    let plan = <RustDetector as super::Detector>::plan(&d, &p, &crate::detect::Verb::Dev).unwrap();
    let cargo_watch_found = which::which("cargo-watch").is_ok();
    if cargo_watch_found {
        assert_eq!(plan.program, "cargo");
        assert!(plan.args.contains(&"watch".to_string()));
        assert_eq!(plan.reason, "cargo watch: cargo-watch found on PATH");
    } else {
        assert_eq!(plan.program, "cargo");
        assert!(plan.args.contains(&"run".to_string()));
        assert!(!plan.args.contains(&"watch".to_string()));
        assert_eq!(plan.reason, "cargo run (cargo-watch not on PATH)");
    }
}
