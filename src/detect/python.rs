//! Python project detector.

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct PythonDetector;

fn detect_manager(dir: &Path) -> (&'static str, &'static str) {
    if dir.join("uv.lock").exists() {
        return ("uv", "uv.lock found");
    }
    if dir.join("poetry.lock").exists() {
        return ("poetry", "poetry.lock found");
    }
    if dir.join("Pipfile.lock").exists() {
        return ("pipenv", "Pipfile.lock found");
    }

    let pyproject = dir.join("pyproject.toml");
    if pyproject.exists() {
        let content = std::fs::read_to_string(&pyproject).unwrap_or_default();
        if content.contains("[tool.uv]") {
            return ("uv", "[tool.uv] in pyproject.toml");
        }
        if content.contains("[tool.poetry]") {
            return ("poetry", "[tool.poetry] in pyproject.toml");
        }
    }
    ("pip", "requirements.txt or pyproject.toml found")
}

/// Detect Django/FastAPI/Flask for dev command.
fn detect_dev_command(dir: &Path) -> Option<CommandPlan> {
    if dir.join("manage.py").exists() {
        return Some(CommandPlan::simple(
            "python",
            ["manage.py", "runserver"],
            dir.to_path_buf(),
            "manage.py found: Django project",
        ));
    }
    // Heuristic: look for fastapi import in common entry points
    for entry in &["main.py", "app.py", "app/main.py", "src/main.py"] {
        let p = dir.join(entry);
        if p.exists() {
            let content = std::fs::read_to_string(&p).unwrap_or_default();
            if content.contains("fastapi") || content.contains("FastAPI") {
                // guess module name from file path
                let module = entry
                    .replace('/', ".")
                    .replace(".py", "")
                    .replace("src.", "");
                return Some(CommandPlan::simple(
                    "uvicorn",
                    [format!("{module}:app"), "--reload".to_string()],
                    dir.to_path_buf(),
                    "FastAPI detected: uvicorn with --reload",
                ));
            }
            if content.contains("flask") || content.contains("Flask") {
                return Some(CommandPlan::simple(
                    "flask",
                    ["run"],
                    dir.to_path_buf(),
                    "Flask detected: flask run",
                ));
            }
        }
    }
    None
}

impl Detector for PythonDetector {
    fn id(&self) -> &'static str {
        "python"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        let has_marker = dir.join("pyproject.toml").exists()
            || dir.join("requirements.txt").exists()
            || dir.join("Pipfile").exists()
            || dir.join("setup.py").exists();

        if !has_marker {
            return None;
        }

        let (pm, _) = detect_manager(dir);

        Some(Project {
            name: dir_name(dir).to_string(),
            root: dir.to_path_buf(),
            kind: ProjectKind::Python,
            package_manager: Some(pm.to_string()),
            scripts: HashMap::new(),
        })
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let (pm, reason) = detect_manager(&project.root);
        let cwd = project.root.clone();

        match verb {
            Verb::Setup => {
                let plan = match pm {
                    "uv" => CommandPlan::simple("uv", ["sync"], cwd, reason),
                    "poetry" => CommandPlan::simple("poetry", ["install"], cwd, reason),
                    "pipenv" => CommandPlan::simple("pipenv", ["install"], cwd, reason),
                    _ => CommandPlan::simple(
                        "pip",
                        ["install", "-r", "requirements.txt"],
                        cwd,
                        reason,
                    ),
                };
                Some(plan)
            }
            Verb::Dev => detect_dev_command(&project.root),
            Verb::Test => {
                let plan = match pm {
                    "uv" => CommandPlan::simple("uv", ["run", "pytest"], cwd, "uv run pytest"),
                    "poetry" => {
                        CommandPlan::simple("poetry", ["run", "pytest"], cwd, "poetry run pytest")
                    }
                    _ => CommandPlan::simple("pytest", [] as [&str; 0], cwd, "pytest"),
                };
                Some(plan)
            }
            Verb::Build => None, // Python has no universal build step
            Verb::Lint => {
                let plan = match pm {
                    "uv" => CommandPlan::simple(
                        "uv",
                        ["run", "ruff", "check", "."],
                        cwd.clone(),
                        "ruff check via uv",
                    )
                    .with_fallback(CommandPlan::simple(
                        "ruff",
                        ["check", "."],
                        cwd,
                        "ruff check",
                    )),
                    _ => CommandPlan::simple("ruff", ["check", "."], cwd.clone(), "ruff check")
                        .with_fallback(CommandPlan::simple(
                            "flake8",
                            [] as [&str; 0],
                            cwd,
                            "flake8 (ruff not found)",
                        )),
                };
                Some(plan)
            }
            Verb::Fmt => {
                let plan = match pm {
                    "uv" => CommandPlan::simple(
                        "uv",
                        ["run", "ruff", "format", "."],
                        cwd.clone(),
                        "ruff format via uv",
                    )
                    .with_fallback(CommandPlan::simple(
                        "ruff",
                        ["format", "."],
                        cwd,
                        "ruff format",
                    )),
                    _ => CommandPlan::simple("ruff", ["format", "."], cwd.clone(), "ruff format")
                        .with_fallback(CommandPlan::simple(
                            "black",
                            ["."],
                            cwd,
                            "black (ruff not found)",
                        )),
                };
                Some(plan)
            }
            Verb::Run(script) => Some(CommandPlan::simple(
                "python",
                [script.as_str()],
                cwd,
                "python script",
            )),
        }
    }

    fn required_tools(&self, project: &Project) -> Vec<ToolRequirement> {
        let pm = project.package_manager.as_deref().unwrap_or("pip");
        let mut tools = vec![ToolRequirement {
            tool: "python".to_string(),
            version_req: ">=3.8".to_string(),
            install_hint: InstallHint {
                brew: Some("python".to_string()),
                apt: Some("python3".to_string()),
                winget: Some("Python.Python.3".to_string()),
                url: Some("https://python.org".to_string()),
            },
        }];
        if pm != "pip" {
            tools.push(ToolRequirement {
                tool: pm.to_string(),
                version_req: String::new(),
                install_hint: InstallHint::brew(pm),
            });
        }
        tools
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
    fn detects_uv() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("pyproject.toml"), "[project]\nname=\"app\"").unwrap();
        fs::write(dir.join("uv.lock"), "").unwrap();

        let d = PythonDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.package_manager.as_deref(), Some("uv"));
    }

    #[test]
    fn detects_poetry() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("pyproject.toml"), "").unwrap();
        fs::write(dir.join("poetry.lock"), "").unwrap();

        let d = PythonDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.package_manager.as_deref(), Some("poetry"));
    }

    #[test]
    fn detects_requirements_txt() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("requirements.txt"), "flask").unwrap();

        let d = PythonDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::Python);
    }

    #[test]
    fn plan_test_uv() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("pyproject.toml"), "").unwrap();
        fs::write(dir.join("uv.lock"), "").unwrap();

        let d = PythonDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Test).unwrap();
        assert_eq!(plan.program, "uv");
        assert!(plan.args.contains(&"pytest".to_string()));
    }

    #[test]
    fn plan_dev_django() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("requirements.txt"), "django").unwrap();
        fs::write(dir.join("manage.py"), "").unwrap();

        let d = PythonDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Dev).unwrap();
        assert_eq!(plan.program, "python");
        assert!(plan.args.contains(&"runserver".to_string()));
    }

    #[test]
    fn no_marker_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = PythonDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
