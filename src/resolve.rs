//! Resolves verbs to command plans using detectors and config overrides.

use anyhow::{bail, Result};
use std::path::Path;

use crate::config::{self, Config};
use crate::detect::{all_detectors, CommandPlan, Project, Verb};

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
    // If config override exists, use it for all detected projects.
    if let Some(cfg) = cfg {
        if let Some(plans) = config::override_for_verb(cfg, verb, dir) {
            // Find or synthesize a project for the name.
            let name = cfg
                .project
                .name
                .clone()
                .unwrap_or_else(|| dir_name(dir).to_string());
            // Try detector for the real project info.
            let project = detect_one_in(dir).unwrap_or_else(|| crate::detect::Project {
                name: name.clone(),
                root: dir.to_path_buf(),
                kind: crate::detect::ProjectKind::Node, // placeholder
                package_manager: None,
                scripts: Default::default(),
            });
            return Ok(vec![Resolution { project, plans }]);
        }
    }

    let detectors = all_detectors();
    let mut results = Vec::new();

    for detector in &detectors {
        if let Some(project) = detector.detect(dir) {
            if let Some(filter) = project_filter {
                if project.name != filter {
                    continue;
                }
            }
            if let Some(plan) = detector.plan(&project, verb) {
                results.push(Resolution {
                    project,
                    plans: vec![plan],
                });
                break; // Primary detector wins
            }
        }
    }

    if results.is_empty() {
        bail!("no command found for verb '{}' in {}", verb, dir.display());
    }

    Ok(results)
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
        if let Some(project) = detector.detect(dir) {
            if let Some(plan) = detector.plan(&project, verb) {
                let label = format!("{}: {}", detector.id(), plan.reason.clone());
                out.push((label, plan));
                break;
            }
        }
    }

    out
}

fn detect_one_in(dir: &Path) -> Option<crate::detect::Project> {
    all_detectors().iter().find_map(|d| d.detect(dir))
}

fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}
