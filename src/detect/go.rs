//! Go project detector.

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct GoDetector;

impl Detector for GoDetector {
    fn id(&self) -> &'static str {
        "go"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        if !dir.join("go.mod").exists() {
            return None;
        }
        let content = std::fs::read_to_string(dir.join("go.mod")).unwrap_or_default();
        let name = extract_module_name(&content).unwrap_or_else(|| dir_name(dir).to_string());

        Some(Project {
            name,
            root: dir.to_path_buf(),
            kind: ProjectKind::Go,
            package_manager: Some("go".to_string()),
            scripts: HashMap::new(),
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let cwd = project.root.clone();

        let plan = match verb {
            Verb::Setup => CommandPlan::simple(
                "go",
                ["mod", "download"],
                cwd,
                "go mod download: go.mod found",
            ),
            Verb::Dev => CommandPlan::simple("go", ["run", "."], cwd, "go run .: go.mod found"),
            Verb::Test => {
                CommandPlan::simple("go", ["test", "./..."], cwd, "go test ./...: go.mod found")
            }
            Verb::Build => CommandPlan::simple(
                "go",
                ["build", "./..."],
                cwd,
                "go build ./...: go.mod found",
            ),
            Verb::Lint => {
                // golangci-lint with go vet fallback
                let primary =
                    CommandPlan::simple("golangci-lint", ["run"], cwd.clone(), "golangci-lint run");
                let fallback = CommandPlan::simple(
                    "go",
                    ["vet", "./..."],
                    cwd,
                    "go vet ./... (golangci-lint not found)",
                );
                primary.with_fallback(fallback)
            }
            Verb::Fmt => CommandPlan::simple("gofmt", ["-w", "."], cwd, "gofmt -w .: go.mod found"),
            Verb::Run(script) => {
                // If a Makefile target exists with this name, let Make handle it under `sgl run`
                if crate::detect::make::detect_make_scripts(&cwd).contains_key(script.as_str()) {
                    return None;
                }
                CommandPlan::simple("go", ["run", script.as_str()], cwd, "go run script")
            }
        };
        Some(plan)
    }

    fn required_tools(&self, _project: &Project) -> Vec<ToolRequirement> {
        vec![ToolRequirement {
            tool: "go".to_string(),
            version_req: String::new(),
            install_hint: InstallHint {
                brew: Some("go".to_string()),
                apt: Some("golang".to_string()),
                winget: Some("GoLang.Go".to_string()),
                url: Some("https://go.dev/dl/".to_string()),
            },
        }]
    }
}

fn extract_module_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("module ") {
            let module = line.strip_prefix("module ")?.trim();
            // Use the last path component as a friendly name
            return Some(module.rsplit('/').next().unwrap_or(module).to_string());
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
    fn detects_go() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("go.mod"),
            "module github.com/foo/myapp\n\ngo 1.21\n",
        )
        .unwrap();

        let d = GoDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::Go);
        assert_eq!(p.name, "myapp");
    }

    #[test]
    fn plan_test() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("go.mod"), "module example.com/app\ngo 1.21\n").unwrap();

        let d = GoDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Test).unwrap();
        assert_eq!(plan.program, "go");
        assert!(plan.args.contains(&"./...".to_string()));
    }

    #[test]
    fn no_go_mod_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = GoDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
