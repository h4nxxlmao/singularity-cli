//! Resolves verbs to command plans using detectors and config overrides.

use anyhow::{bail, Result};
use std::path::Path;

use crate::config::{self, Config};
use crate::detect::{all_detectors, CommandPlan, Detector, Project, ProjectKind, Verb};

/// The result of resolving a verb for a project.
pub struct Resolution {
    pub project: Project,
    /// Primary plan (from config override or detector).
    pub plans: Vec<CommandPlan>,
}

/// Resolve a verb for the given directory.
/// Config overrides take priority over detector plans.
pub fn resolve(
    dir: &Path,
    verb: &Verb,
    project_filter: Option<&str>,
    cfg: Option<&Config>,
) -> Result<Vec<Resolution>> {
    // 1. If config override exists, use it.
    if let Some(cfg) = cfg {
        if let Some(plans) = config::override_for_verb(cfg, verb, dir) {
            let name = cfg
                .project
                .name
                .clone()
                .unwrap_or_else(|| dir_name(dir).to_string());
            let project = detect_one_in(dir).unwrap_or_else(|| crate::detect::Project {
                name: name.clone(),
                root: dir.to_path_buf(),
                kind: crate::detect::ProjectKind::Node,
                package_manager: None,
                scripts: Default::default(),
            });
            return Ok(vec![Resolution { project, plans }]);
        }
    }

    let detectors = all_detectors();
    let mut detected_project: Option<(Project, &'static str)> = None;

    for detector in &detectors {
        if let Some(mut project) = detector.detect(dir) {
            if let Some(filter) = project_filter {
                if project.name != filter {
                    continue;
                }
            }

            // If a Makefile also exists in this directory, merge its targets into project.scripts
            if project.kind != ProjectKind::Make {
                let make_scripts = crate::detect::make::detect_make_scripts(dir);
                for (k, v) in make_scripts {
                    project.scripts.entry(k).or_insert(v);
                }
            }

            // Try detector's native plan for this verb
            if let Some(plan) = detector.plan(&project, verb) {
                return Ok(vec![Resolution {
                    project,
                    plans: vec![plan],
                }]);
            }

            // If this is Verb::Run, and a Makefile has this target, dispatch via make
            if let Verb::Run(target) = verb {
                if let Some((runner, _)) = crate::detect::make::detect_runner(dir) {
                    if project.scripts.contains_key(target) {
                        return Ok(vec![Resolution {
                            project,
                            plans: vec![CommandPlan::simple(
                                runner,
                                [target.as_str()],
                                dir.to_path_buf(),
                                format!("{runner} target: {target}"),
                            )],
                        }]);
                    }
                }
            }

            detected_project = Some((project, detector.id()));
            break; // Primary detector takes ownership of this directory
        }
    }

    let verb_str = match verb {
        Verb::Run(s) => s.as_str(),
        v => v.name(),
    };

    if let Some((project, _)) = detected_project {
        let msg = format_unavailable_message(dir, verb_str, Some(&project));
        bail!("{msg}");
    }

    let msg = format_unavailable_message(dir, verb_str, None);
    bail!("{msg}");
}

/// Resolve a verb and return all available plans with explanations (for --explain).
pub fn explain(dir: &Path, verb: &Verb, cfg: Option<&Config>) -> Vec<(String, CommandPlan)> {
    let mut out = Vec::new();

    if let Some(cfg) = cfg {
        if let Some(plans) = config::override_for_verb(cfg, verb, dir) {
            for plan in plans {
                let label = format!("singularity.toml override: {}", plan.reason);
                out.push((label, plan));
            }
            return out;
        }
    }

    for detector in all_detectors() {
        if let Some(mut project) = detector.detect(dir) {
            // Merge Makefile targets if not Make itself
            if project.kind != ProjectKind::Make {
                let make_scripts = crate::detect::make::detect_make_scripts(dir);
                for (k, v) in make_scripts {
                    project.scripts.entry(k).or_insert(v);
                }
            }

            if let Some(plan) = detector.plan(&project, verb) {
                let label = format!("{}: {}", detector.id(), plan.reason);
                out.push((label, plan));
                break;
            }

            // Check make fallback for run
            if let Verb::Run(target) = verb {
                if let Some((runner, _)) = crate::detect::make::detect_runner(dir) {
                    if project.scripts.contains_key(target) {
                        let label = format!("{runner}: {runner} {target}");
                        out.push((
                            label,
                            CommandPlan::simple(
                                runner,
                                [target.as_str()],
                                dir.to_path_buf(),
                                format!("{runner} target: {target}"),
                            ),
                        ));
                        break;
                    }
                }
            }

            break; // Primary detector wins
        }
    }

    out
}

fn supported_verbs_for_kind(project: &Project) -> Vec<&'static str> {
    match project.kind {
        ProjectKind::Node => vec![
            "setup",
            "dev",
            "test",
            "build",
            "lint",
            "fmt",
            "run <script>",
        ],
        ProjectKind::Python => vec!["setup", "dev", "test", "lint", "fmt", "run <script>"],
        ProjectKind::Rust => vec!["setup", "dev", "test", "build", "lint", "fmt", "run <bin>"],
        ProjectKind::Go => vec![
            "setup",
            "dev",
            "test",
            "build",
            "lint",
            "fmt",
            "run <script>",
        ],
        ProjectKind::Java => vec!["setup", "dev", "build", "test", "lint", "run <task>"],
        ProjectKind::DotNet => vec!["setup", "dev", "build", "test", "fmt", "run"],
        ProjectKind::Ruby => vec!["setup", "dev", "test", "lint", "fmt", "run <script>"],
        ProjectKind::Docker => vec!["setup", "dev", "build", "run <service>"],
        ProjectKind::Make => vec!["run <target>"],
    }
}

pub fn detect_one_in(dir: &Path) -> Option<crate::detect::Project> {
    all_detectors().iter().find_map(|d| d.detect(dir))
}

pub fn format_unavailable_message(dir: &Path, verb_str: &str, project: Option<&Project>) -> String {
    if let Some(project) = project {
        let pm_str = match &project.package_manager {
            Some(pm) if pm != "docker" && pm.as_str() != project.kind.to_string().as_str() => {
                let display_pm = if pm == "mvn" { "maven" } else { pm.as_str() };
                format!("{}, {}", project.kind, display_pm)
            }
            _ => format!("{}", project.kind),
        };
        let reason = match project.kind {
            ProjectKind::Node => {
                if project.scripts.is_empty() {
                    format!("no \"{verb_str}\" script in package.json (no scripts defined in package.json)")
                } else {
                    format!("no \"{verb_str}\" script in package.json")
                }
            }
            ProjectKind::Docker => {
                format!("docker compose does not define a standard '{verb_str}' command")
            }
            ProjectKind::Java => {
                if verb_str == "lint" {
                    if project.package_manager.as_deref() == Some("gradle") {
                        "neither checkstyle, spotless, nor ktlint plugins are configured in build.gradle".to_string()
                    } else {
                        "neither checkstyle nor spotless plugins are configured in pom.xml"
                            .to_string()
                    }
                } else {
                    format!("verb '{verb_str}' is not supported by {}", pm_str)
                }
            }
            _ => format!(
                "verb '{verb_str}' is not supported for {} projects",
                project.kind
            ),
        };

        let available = match project.kind {
            ProjectKind::Node => {
                if project.scripts.is_empty() {
                    "none".to_string()
                } else {
                    let mut scripts: Vec<&str> =
                        project.scripts.keys().map(|s| s.as_str()).collect();
                    scripts.sort_unstable();
                    scripts.join(", ")
                }
            }
            ProjectKind::Docker => "setup, dev, build".to_string(),
            ProjectKind::Java => {
                let has_lint = if verb_str == "lint" {
                    false
                } else {
                    crate::detect::java::JavaDetector
                        .plan(project, &Verb::Lint)
                        .is_some()
                };
                if has_lint {
                    "setup, dev, test, build, lint".to_string()
                } else {
                    "setup, dev, test, build".to_string()
                }
            }
            _ => {
                let verbs = supported_verbs_for_kind(project);
                verbs.join(", ")
            }
        };

        let hint = match project.kind {
            ProjectKind::Node => format!(
                "add a \"{verb_str}\" script to package.json, or define it in singularity.toml under [commands]"
            ),
            ProjectKind::Docker => format!(
                "run custom container commands with docker compose run, or define '{verb_str}' in singularity.toml under [commands]"
            ),
            ProjectKind::Java if verb_str == "lint" => {
                if project.package_manager.as_deref() == Some("gradle") {
                    "configure a checkstyle, spotless, or ktlint plugin in build.gradle, or define 'lint' in singularity.toml under [commands]".to_string()
                } else {
                    "configure a checkstyle or spotless plugin in pom.xml, or define 'lint' in singularity.toml under [commands]".to_string()
                }
            }
            _ => format!("define '{verb_str}' in singularity.toml under [commands]"),
        };

        format!(
            "sgl: no '{verb_str}' command for this project ({pm_str})\npath: {}\nreason: {reason}\navailable: {available}\nhint: {hint}",
            project.root.display()
        )
    } else {
        format!(
            "sgl: no project detected in {}\navailable: setup, dev, test, build, lint, fmt, run <script>, doctor, info, init\nhint: run 'sgl init' to create a singularity.toml",
            dir.display()
        )
    }
}

fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}
