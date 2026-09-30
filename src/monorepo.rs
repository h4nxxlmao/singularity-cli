//! Monorepo discovery: walk the tree and find all sub-projects.

use std::path::Path;

use ignore::WalkBuilder;

use crate::detect::{all_detectors, Project};

/// Walk `root` up to `max_depth` levels, respecting .gitignore,
/// and return all detected sub-projects.
pub fn discover(root: &Path) -> Vec<Project> {
    let mut projects: Vec<Project> = Vec::new();
    let skip_dirs = ["node_modules", "target", ".venv", "dist", ".git"];

    let walker = WalkBuilder::new(root)
        .max_depth(Some(4))
        .hidden(false)
        .git_ignore(true)
        .filter_entry(move |entry| {
            let name = entry.file_name().to_string_lossy();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                !skip_dirs.contains(&name.as_ref())
            } else {
                true
            }
        })
        .build();

    let detectors = all_detectors();

    for entry in walker.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let dir = entry.path();
        // Skip root itself (already checked by caller if needed)
        if dir == root {
            continue;
        }

        for detector in &detectors {
            if let Some(project) = detector.detect(dir) {
                // Avoid duplicates (same root)
                if !projects.iter().any(|p| p.root == project.root) {
                    projects.push(project);
                }
                break;
            }
        }
    }

    projects
}

/// Returns true if `dir` contains more than one detectable project.
pub fn is_monorepo(root: &Path) -> bool {
    discover(root).len() > 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discovers_sub_projects() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        // Create two sub-projects
        let web = root.join("web");
        fs::create_dir_all(&web).unwrap();
        fs::write(
            web.join("package.json"),
            r#"{"name":"web","scripts":{"build":"vite build"}}"#,
        )
        .unwrap();

        let api = root.join("api");
        fs::create_dir_all(&api).unwrap();
        fs::write(
            api.join("Cargo.toml"),
            "[package]\nname=\"api\"\nversion=\"0.1.0\"",
        )
        .unwrap();

        let found = discover(root);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn skips_node_modules() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let nm = root.join("node_modules").join("some-pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("package.json"), r#"{"name":"some-pkg"}"#).unwrap();

        let found = discover(root);
        assert!(found.is_empty());
    }
}
