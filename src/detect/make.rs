//! Makefile / justfile / Taskfile detector.

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, Project, ProjectKind, ToolRequirement, Verb};

pub struct MakeDetector;

fn parse_make_targets(content: &str) -> HashMap<String, String> {
    let mut targets = HashMap::new();
    for line in content.lines() {
        // A target line: starts with a non-whitespace char, has a colon, no leading tab.
        if line.starts_with('\t') || line.starts_with(' ') || line.is_empty() {
            continue;
        }
        if let Some(colon_pos) = line.find(':') {
            let target = line[..colon_pos].trim();
            // Skip variables (containing =) and phony markers
            if !target.contains('=') && !target.starts_with('.') {
                targets.insert(target.to_string(), format!("make {target}"));
            }
        }
    }
    targets
}

fn detect_runner(dir: &Path) -> Option<(&'static str, &'static str)> {
    // (runner, marker file)
    if dir.join("Taskfile.yml").exists() || dir.join("Taskfile.yaml").exists() {
        return Some(("task", "Taskfile.yml"));
    }
    if dir.join("justfile").exists() || dir.join("Justfile").exists() {
        return Some(("just", "justfile"));
    }
    if dir.join("Makefile").exists() || dir.join("makefile").exists() {
        return Some(("make", "Makefile"));
    }
    None
}

impl Detector for MakeDetector {
    fn id(&self) -> &'static str {
        "make"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        let (runner, marker) = detect_runner(dir)?;

        let scripts = if runner == "make" {
            let content = std::fs::read_to_string(dir.join(marker)).unwrap_or_default();
            parse_make_targets(&content)
        } else {
            HashMap::new()
        };

        Some(Project {
            name: dir_name(dir).to_string(),
            root: dir.to_path_buf(),
            kind: ProjectKind::Make,
            package_manager: Some(runner.to_string()),
            scripts,
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let runner = project.package_manager.as_deref().unwrap_or("make");
        let cwd = project.root.clone();

        match verb {
            Verb::Run(target) => Some(CommandPlan::simple(
                runner,
                [target.as_str()],
                cwd,
                format!("{runner} target"),
            )),
            // Map standard verbs to same-named targets if they exist
            other => {
                let target_name = other.name();
                if project.scripts.contains_key(target_name) {
                    Some(CommandPlan::simple(
                        runner,
                        [target_name],
                        cwd,
                        format!("{runner}: {target_name} target found"),
                    ))
                } else {
                    None
                }
            }
        }
    }

    fn required_tools(&self, project: &Project) -> Vec<ToolRequirement> {
        let runner = project.package_manager.as_deref().unwrap_or("make");
        vec![ToolRequirement {
            tool: runner.to_string(),
            version_req: String::new(),
            install_hint: super::InstallHint {
                brew: Some(runner.to_string()),
                apt: Some(runner.to_string()),
                winget: Some(match runner {
                    "just" => "Casey.Just".to_string(),
                    "task" => "Task.Task".to_string(),
                    _ => "GnuWin32.Make".to_string(),
                }),
                url: None,
            },
        }]
    }
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
    fn detects_makefile() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("Makefile"),
            "test:\n\tcargo test\nbuild:\n\tcargo build\n",
        )
        .unwrap();

        let d = MakeDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::Make);
        assert!(p.scripts.contains_key("test"));
        assert!(p.scripts.contains_key("build"));
    }

    #[test]
    fn plan_run_target() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Makefile"), "deploy:\n\t./deploy.sh\n").unwrap();

        let d = MakeDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Run("deploy".into())).unwrap();
        assert_eq!(plan.program, "make");
        assert!(plan.args.contains(&"deploy".to_string()));
    }

    #[test]
    fn maps_verb_to_target() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("Makefile"), "test:\n\tgo test ./...\n").unwrap();

        let d = MakeDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Test).unwrap();
        assert_eq!(plan.program, "make");
        assert!(plan.args.contains(&"test".to_string()));
    }

    #[test]
    fn no_makefile_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = MakeDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
