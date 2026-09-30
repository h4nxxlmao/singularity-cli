//! Node.js project detector.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct NodeDetector;

/// Subset of package.json we care about.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct PackageJson {
    name: Option<String>,
    package_manager: Option<String>,
    scripts: HashMap<String, String>,
    engines: Option<Engines>,
    workspaces: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
struct Engines {
    node: Option<String>,
}

fn detect_package_manager(dir: &Path, pkg: &PackageJson) -> String {
    // Lockfile takes precedence
    if dir.join("pnpm-lock.yaml").exists() {
        return "pnpm".to_string();
    }
    if dir.join("yarn.lock").exists() {
        return "yarn".to_string();
    }
    if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() {
        return "bun".to_string();
    }
    if dir.join("package-lock.json").exists() {
        return "npm".to_string();
    }
    // "packageManager" field
    if let Some(pm) = &pkg.package_manager {
        let name = pm.split('@').next().unwrap_or("npm");
        return name.to_string();
    }
    "npm".to_string()
}

impl Detector for NodeDetector {
    fn id(&self) -> &'static str {
        "node"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        let pkg_path = dir.join("package.json");
        if !pkg_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&pkg_path).ok()?;
        let pkg: PackageJson = serde_json::from_str(&content).unwrap_or_default();

        let pm = detect_package_manager(dir, &pkg);
        let name = pkg
            .name
            .clone()
            .unwrap_or_else(|| dir_name(dir).to_string());

        Some(Project {
            name,
            root: dir.to_path_buf(),
            kind: ProjectKind::Node,
            package_manager: Some(pm),
            scripts: pkg.scripts,
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let pm = project.package_manager.as_deref().unwrap_or("npm");
        let cwd = project.root.clone();

        let plan = match verb {
            Verb::Setup => {
                CommandPlan::simple(pm, ["install"], cwd, format!("{pm}: install dependencies"))
            }
            Verb::Dev => {
                // Prefer "dev" script, fall back to "start"
                if project.scripts.contains_key("dev") {
                    run_script(pm, "dev", cwd, "package.json scripts.dev")
                } else if project.scripts.contains_key("start") {
                    run_script(
                        pm,
                        "start",
                        cwd,
                        "package.json scripts.start (dev fallback)",
                    )
                } else {
                    return None;
                }
            }
            Verb::Test => {
                if project.scripts.contains_key("test") {
                    run_script(pm, "test", cwd, "package.json scripts.test")
                } else {
                    return None;
                }
            }
            Verb::Build => {
                if project.scripts.contains_key("build") {
                    run_script(pm, "build", cwd, "package.json scripts.build")
                } else {
                    return None;
                }
            }
            Verb::Lint => {
                if project.scripts.contains_key("lint") {
                    run_script(pm, "lint", cwd, "package.json scripts.lint")
                } else {
                    return None;
                }
            }
            Verb::Fmt => {
                // Prefer "format", fall back to "fmt"
                if project.scripts.contains_key("format") {
                    run_script(pm, "format", cwd, "package.json scripts.format")
                } else if project.scripts.contains_key("fmt") {
                    run_script(pm, "fmt", cwd, "package.json scripts.fmt")
                } else {
                    return None;
                }
            }
            Verb::Run(script) => {
                if project.scripts.contains_key(script.as_str()) {
                    run_script(
                        pm,
                        script.as_str(),
                        cwd,
                        "package.json scripts custom target",
                    )
                } else {
                    return None;
                }
            }
        };
        Some(plan)
    }

    fn required_tools(&self, project: &Project) -> Vec<ToolRequirement> {
        let pm = project.package_manager.as_deref().unwrap_or("npm");
        vec![
            ToolRequirement {
                tool: "node".to_string(),
                version_req: String::new(),
                install_hint: InstallHint {
                    brew: Some("node".to_string()),
                    apt: Some("nodejs".to_string()),
                    winget: Some("OpenJS.NodeJS".to_string()),
                    url: Some("https://nodejs.org".to_string()),
                },
            },
            ToolRequirement {
                tool: pm.to_string(),
                version_req: String::new(),
                install_hint: InstallHint::brew(pm),
            },
        ]
    }
}

fn run_script(pm: &str, script: &str, cwd: std::path::PathBuf, reason: &str) -> CommandPlan {
    let args: Vec<String> = match pm {
        "npm" => vec!["run".into(), script.into()],
        "pnpm" => vec!["run".into(), script.into()],
        "yarn" => vec![script.into()],
        "bun" => vec!["run".into(), script.into()],
        _ => vec!["run".into(), script.into()],
    };
    CommandPlan::simple(pm, args, cwd, reason)
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

    fn make_pkg(dir: &Path, content: &str) {
        fs::write(dir.join("package.json"), content).unwrap();
    }

    #[test]
    fn detects_npm() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        make_pkg(
            dir,
            r#"{"name":"myapp","scripts":{"test":"jest","build":"tsc"}}"#,
        );
        fs::write(dir.join("package-lock.json"), "{}").unwrap();

        let d = NodeDetector;
        let project = d.detect(dir).unwrap();
        assert_eq!(project.package_manager.as_deref(), Some("npm"));
        assert_eq!(project.kind, ProjectKind::Node);
    }

    #[test]
    fn detects_pnpm() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        make_pkg(dir, r#"{"name":"myapp","scripts":{"dev":"vite"}}"#);
        fs::write(dir.join("pnpm-lock.yaml"), "").unwrap();

        let d = NodeDetector;
        let project = d.detect(dir).unwrap();
        assert_eq!(project.package_manager.as_deref(), Some("pnpm"));
    }

    #[test]
    fn detects_yarn() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        make_pkg(dir, r#"{"scripts":{"test":"jest"}}"#);
        fs::write(dir.join("yarn.lock"), "").unwrap();

        let d = NodeDetector;
        let project = d.detect(dir).unwrap();
        assert_eq!(project.package_manager.as_deref(), Some("yarn"));
    }

    #[test]
    fn detects_bun() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        make_pkg(dir, r#"{"scripts":{"dev":"bun run src/index.ts"}}"#);
        fs::write(dir.join("bun.lock"), "").unwrap();

        let d = NodeDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.package_manager.as_deref(), Some("bun"));
    }

    #[test]
    fn plan_dev_falls_back_to_start() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        make_pkg(dir, r#"{"scripts":{"start":"node index.js"}}"#);

        let d = NodeDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Dev).unwrap();
        assert!(plan.args.contains(&"start".to_string()));
    }

    #[test]
    fn plan_fmt_falls_back_to_fmt_script() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        make_pkg(dir, r#"{"scripts":{"fmt":"prettier --write ."}}"#);

        let d = NodeDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Fmt).unwrap();
        assert!(plan.args.contains(&"fmt".to_string()));
    }

    #[test]
    fn no_package_json_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = NodeDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
