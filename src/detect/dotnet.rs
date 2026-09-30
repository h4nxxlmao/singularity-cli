//! .NET project detector.

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct DotNetDetector;

impl Detector for DotNetDetector {
    fn id(&self) -> &'static str {
        "dotnet"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        let has_sln = glob_exists(dir, "*.sln");
        let has_csproj = glob_exists(dir, "*.csproj");

        if !has_sln && !has_csproj {
            return None;
        }

        Some(Project {
            name: dir_name(dir).to_string(),
            root: dir.to_path_buf(),
            kind: ProjectKind::DotNet,
            package_manager: Some("dotnet".to_string()),
            scripts: HashMap::new(),
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let cwd = project.root.clone();

        let plan = match verb {
            Verb::Setup => CommandPlan::simple(
                "dotnet",
                ["restore"],
                cwd,
                "dotnet restore: .sln/.csproj found",
            ),
            Verb::Build => CommandPlan::simple("dotnet", ["build"], cwd, "dotnet build"),
            Verb::Test => CommandPlan::simple("dotnet", ["test"], cwd, "dotnet test"),
            Verb::Run(_) => CommandPlan::simple("dotnet", ["run"], cwd, "dotnet run"),
            Verb::Dev => {
                let csproj_count = std::fs::read_dir(&cwd)
                    .ok()
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().is_some_and(|ext| ext == "csproj"))
                            .count()
                    })
                    .unwrap_or(0);

                let sln_count = std::fs::read_dir(&cwd)
                    .ok()
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .find(|e| e.path().extension().is_some_and(|ext| ext == "sln"))
                    })
                    .unwrap_or(None);

                let projects = if let Some(sln) = sln_count {
                    let content = std::fs::read_to_string(sln.path()).unwrap_or_default();
                    content
                        .lines()
                        .filter(|l| l.contains("Project(") && l.contains(".csproj"))
                        .count()
                } else {
                    csproj_count
                };

                if projects == 1 {
                    CommandPlan::simple("dotnet", ["run"], cwd, "dotnet run: single project")
                } else {
                    return None;
                }
            }
            Verb::Fmt => CommandPlan::simple("dotnet", ["format"], cwd, "dotnet format"),
            _ => return None,
        };
        Some(plan)
    }

    fn required_tools(&self, _project: &Project) -> Vec<ToolRequirement> {
        vec![ToolRequirement {
            tool: "dotnet".to_string(),
            version_req: String::new(),
            install_hint: InstallHint {
                brew: Some("dotnet".to_string()),
                apt: Some("dotnet-sdk-8.0".to_string()),
                winget: Some("Microsoft.DotNet.SDK.8".to_string()),
                url: Some("https://dotnet.microsoft.com/download".to_string()),
            },
        }]
    }
}

fn glob_exists(dir: &Path, pattern: &str) -> bool {
    let ext = pattern.trim_start_matches("*.");
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(ext) {
                    return true;
                }
            }
        }
    }
    false
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
    fn detects_sln() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("MyApp.sln"), "").unwrap();

        let d = DotNetDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::DotNet);
    }

    #[test]
    fn detects_csproj() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("MyApp.csproj"), "<Project/>").unwrap();

        let d = DotNetDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::DotNet);
    }

    #[test]
    fn no_dotnet_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = DotNetDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
#[cfg(test)]
mod dotnet_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn single_csproj_dev_resolves_to_dotnet_run() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("App.csproj"),
            "<Project Sdk=\"Microsoft.NET.Sdk\"/>",
        )
        .unwrap();
        let d = DotNetDetector;
        let p = d.detect(tmp.path()).unwrap();
        let plan = d.plan(&p, &Verb::Dev).unwrap();
        assert_eq!(plan.program, "dotnet");
        assert!(plan.args.contains(&"run".to_string()));
    }

    #[test]
    fn multi_csproj_dev_returns_none() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("App1.csproj"), "<Project/>").unwrap();
        fs::write(tmp.path().join("App2.csproj"), "<Project/>").unwrap();
        let d = DotNetDetector;
        let p = d.detect(tmp.path()).unwrap();
        let plan = d.plan(&p, &Verb::Dev);
        assert!(plan.is_none(), "multi-csproj should not resolve dev");
    }
}

