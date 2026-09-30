//! Docker Compose detector.

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct DockerDetector;

impl Detector for DockerDetector {
    fn id(&self) -> &'static str {
        "docker"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        let has_compose = dir.join("docker-compose.yml").exists()
            || dir.join("docker-compose.yaml").exists()
            || dir.join("compose.yml").exists()
            || dir.join("compose.yaml").exists();

        if !has_compose {
            return None;
        }

        Some(Project {
            name: dir_name(dir).to_string(),
            root: dir.to_path_buf(),
            kind: ProjectKind::Docker,
            package_manager: Some("docker".to_string()),
            scripts: HashMap::new(),
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let cwd = project.root.clone();

        let plan = match verb {
            Verb::Setup => CommandPlan::simple(
                "docker",
                ["compose", "pull"],
                cwd,
                "docker compose pull: compose file found",
            ),
            Verb::Dev => CommandPlan::simple(
                "docker",
                ["compose", "up"],
                cwd,
                "docker compose up: compose file found",
            ),
            Verb::Build => CommandPlan::simple(
                "docker",
                ["compose", "build"],
                cwd,
                "docker compose build: compose file found",
            ),
            Verb::Run(service) => CommandPlan::simple(
                "docker",
                ["compose", "run", "--rm", service.as_str()],
                cwd,
                "docker compose run",
            ),
            _ => return None,
        };
        Some(plan)
    }

    fn required_tools(&self, _project: &Project) -> Vec<ToolRequirement> {
        vec![ToolRequirement {
            tool: "docker".to_string(),
            version_req: String::new(),
            install_hint: InstallHint {
                brew: Some("docker".to_string()),
                apt: Some("docker.io".to_string()),
                winget: Some("Docker.DockerDesktop".to_string()),
                url: Some("https://docs.docker.com/get-docker/".to_string()),
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
    fn detects_compose_yml() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("docker-compose.yml"), "version: '3'").unwrap();

        let d = DockerDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::Docker);
    }

    #[test]
    fn plan_dev() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("compose.yaml"), "").unwrap();

        let d = DockerDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Dev).unwrap();
        assert_eq!(plan.program, "docker");
        assert!(plan.args.contains(&"up".to_string()));
    }

    #[test]
    fn no_compose_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = DockerDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
