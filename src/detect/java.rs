//! Java project detector (Maven and Gradle).

use std::collections::HashMap;
use std::path::Path;

use super::{CommandPlan, Detector, InstallHint, Project, ProjectKind, ToolRequirement, Verb};

pub struct JavaDetector;

fn gradlew(dir: &Path) -> &'static str {
    if cfg!(windows) {
        if dir.join("gradlew.bat").exists() {
            return "gradlew.bat";
        }
    } else if dir.join("gradlew").exists() {
        return "./gradlew";
    }
    "gradle"
}

impl Detector for JavaDetector {
    fn id(&self) -> &'static str {
        "java"
    }

    fn detect(&self, dir: &Path) -> Option<Project> {
        if dir.join("pom.xml").exists() {
            return Some(Project {
                name: dir_name(dir).to_string(),
                root: dir.to_path_buf(),
                kind: ProjectKind::Java,
                package_manager: Some("mvn".to_string()),
                scripts: HashMap::new(),
            });
        }
        let has_gradle = dir.join("build.gradle").exists() || dir.join("build.gradle.kts").exists();
        if has_gradle {
            let pm = gradlew(dir);
            return Some(Project {
                name: dir_name(dir).to_string(),
                root: dir.to_path_buf(),
                kind: ProjectKind::Java,
                package_manager: Some(pm.to_string()),
                scripts: HashMap::new(),
            });
        }
        None
    }

    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan> {
        let pm = project.package_manager.as_deref().unwrap_or("mvn");
        let cwd = project.root.clone();

        let plan = match (pm, verb) {
            ("mvn", Verb::Setup) => CommandPlan::simple(
                "mvn",
                ["dependency:resolve"],
                cwd,
                "mvn dependency:resolve: pom.xml found",
            ),
            ("mvn", Verb::Build) => CommandPlan::simple(
                "mvn",
                ["package", "-DskipTests"],
                cwd,
                "mvn package: pom.xml found",
            ),
            ("mvn", Verb::Test) => {
                CommandPlan::simple("mvn", ["test"], cwd, "mvn test: pom.xml found")
            }
            ("mvn", Verb::Lint) => maven_lint_plan(&cwd)?,
            ("mvn", Verb::Fmt) => {
                let pom = cwd.join("pom.xml");
                let content = std::fs::read_to_string(&pom).ok()?;
                if content.contains("spotless") {
                    CommandPlan::simple("mvn", ["spotless:apply"], cwd, "mvn spotless:apply")
                } else {
                    return None;
                }
            }
            (_, Verb::Setup) => CommandPlan::simple(
                pm,
                ["dependencies"],
                cwd,
                "gradle dependencies: build.gradle found",
            ),
            (_, Verb::Build) => {
                CommandPlan::simple(pm, ["build"], cwd, "gradle build: build.gradle found")
            }
            (_, Verb::Test) => {
                CommandPlan::simple(pm, ["test"], cwd, "gradle test: build.gradle found")
            }
            (_, Verb::Lint) => gradle_lint_plan(pm, &cwd)?,
            (_, Verb::Fmt) => {
                let mut found_spotless = false;
                for file in &["build.gradle", "build.gradle.kts"] {
                    if let Ok(c) = std::fs::read_to_string(cwd.join(file)) {
                        if c.contains("spotless") {
                            found_spotless = true;
                            break;
                        }
                    }
                }
                if found_spotless {
                    CommandPlan::simple(pm, ["spotlessApply"], cwd, "gradle spotlessApply")
                } else {
                    return None;
                }
            }
            (_, Verb::Run(task)) => CommandPlan::simple(pm, [task.as_str()], cwd, "gradle task"),
            _ => return None,
        };
        Some(plan)
    }

    fn required_tools(&self, project: &Project) -> Vec<ToolRequirement> {
        let pm = project.package_manager.as_deref().unwrap_or("mvn");
        vec![ToolRequirement {
            tool: pm.trim_start_matches("./").to_string(),
            version_req: String::new(),
            install_hint: InstallHint {
                brew: Some(
                    if pm.contains("mvn") {
                        "maven"
                    } else {
                        "gradle"
                    }
                    .to_string(),
                ),
                apt: Some(
                    if pm.contains("mvn") {
                        "maven"
                    } else {
                        "gradle"
                    }
                    .to_string(),
                ),
                winget: Some(
                    if pm.contains("mvn") {
                        "Apache.Maven"
                    } else {
                        "Gradle.Gradle"
                    }
                    .to_string(),
                ),
                url: None,
            },
        }]
    }
}

fn maven_lint_plan(cwd: &Path) -> Option<CommandPlan> {
    let pom = cwd.join("pom.xml");
    let content = std::fs::read_to_string(&pom).ok()?;
    if content.contains("spotless") {
        Some(CommandPlan::simple(
            "mvn",
            ["spotless:check"],
            cwd.to_path_buf(),
            "mvn spotless:check configured in pom.xml",
        ))
    } else if content.contains("checkstyle") {
        Some(CommandPlan::simple(
            "mvn",
            ["checkstyle:check"],
            cwd.to_path_buf(),
            "mvn checkstyle:check configured in pom.xml",
        ))
    } else {
        None
    }
}

fn gradle_lint_plan(pm: &str, cwd: &Path) -> Option<CommandPlan> {
    for file in &["build.gradle", "build.gradle.kts"] {
        if let Ok(content) = std::fs::read_to_string(cwd.join(file)) {
            if content.contains("spotless") {
                return Some(CommandPlan::simple(
                    pm,
                    ["spotlessCheck"],
                    cwd.to_path_buf(),
                    "gradle spotlessCheck configured",
                ));
            }
            if content.contains("checkstyle") {
                return Some(CommandPlan::simple(
                    pm,
                    ["checkstyleMain"],
                    cwd.to_path_buf(),
                    "gradle checkstyleMain configured",
                ));
            }
            if content.contains("ktlint") {
                return Some(CommandPlan::simple(
                    pm,
                    ["ktlintCheck"],
                    cwd.to_path_buf(),
                    "gradle ktlintCheck configured",
                ));
            }
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
    fn detects_maven() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("pom.xml"), "<project/>").unwrap();

        let d = JavaDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.package_manager.as_deref(), Some("mvn"));
    }

    #[test]
    fn detects_gradle() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(dir.join("build.gradle"), "").unwrap();

        let d = JavaDetector;
        let p = d.detect(dir).unwrap();
        assert_eq!(p.kind, ProjectKind::Java);
    }

    #[test]
    fn maven_lint_not_available_without_checkstyle_or_spotless() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("pom.xml"),
            "<project><groupId>test</groupId></project>",
        )
        .unwrap();

        let d = JavaDetector;
        let p = d.detect(dir).unwrap();
        assert!(d.plan(&p, &Verb::Lint).is_none());
    }

    #[test]
    fn maven_lint_available_with_checkstyle() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("pom.xml"),
            "<project><plugins><plugin><artifactId>maven-checkstyle-plugin</artifactId></plugin></plugins></project>",
        )
        .unwrap();

        let d = JavaDetector;
        let p = d.detect(dir).unwrap();
        let plan = d.plan(&p, &Verb::Lint).unwrap();
        assert_eq!(plan.program, "mvn");
        assert!(plan.args.contains(&"checkstyle:check".to_string()));
    }

    #[test]
    fn no_java_returns_none() {
        let tmp = TempDir::new().unwrap();
        let d = JavaDetector;
        assert!(d.detect(tmp.path()).is_none());
    }
}
