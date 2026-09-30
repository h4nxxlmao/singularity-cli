//! Git helpers for monorepo --changed support.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Return the files changed against `base_branch` (default: main/master).
pub fn changed_files(repo_root: &Path, base_branch: Option<&str>) -> Vec<PathBuf> {
    let base = base_branch.unwrap_or("main");

    // Try `git diff --name-only <base>...HEAD`
    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("{base}...HEAD")])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| repo_root.join(l.trim()))
            .collect(),
        _ => {
            // Fallback: uncommitted changes
            let output2 = Command::new("git")
                .args(["diff", "--name-only"])
                .current_dir(repo_root)
                .output();
            match output2 {
                Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(|l| repo_root.join(l.trim()))
                    .collect(),
                _ => vec![],
            }
        }
    }
}

/// Return which of `projects` have files in the changed set.
pub fn filter_changed<'a>(
    projects: &'a [crate::detect::Project],
    changed: &[PathBuf],
) -> Vec<&'a crate::detect::Project> {
    projects
        .iter()
        .filter(|p| changed.iter().any(|f| f.starts_with(&p.root)))
        .collect()
}

/// Find the git repository root for the given directory, if any.
pub fn repo_root(dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);
        Some(PathBuf::from(s.trim()))
    } else {
        None
    }
}
