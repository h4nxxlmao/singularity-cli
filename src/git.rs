//! Git helpers for monorepo --changed support.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Return the files changed against `base_branch` (default: main/master)
/// plus uncommitted and untracked changes.
pub fn changed_files(repo_root: &Path, base_branch: Option<&str>) -> Vec<PathBuf> {
    let base = base_branch.unwrap_or("main");
    let mut files = Vec::new();

    // 1. git diff --name-only <base>...HEAD
    if let Ok(out) = Command::new("git")
        .args(["diff", "--name-only", &format!("{base}...HEAD")])
        .current_dir(repo_root)
        .output()
    {
        if out.status.success() {
            for l in String::from_utf8_lossy(&out.stdout).lines() {
                let trimmed = l.trim();
                if !trimmed.is_empty() {
                    files.push(repo_root.join(trimmed));
                }
            }
        }
    }

    // 2. git diff --name-only (unstaged changes)
    if let Ok(out) = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(repo_root)
        .output()
    {
        if out.status.success() {
            for l in String::from_utf8_lossy(&out.stdout).lines() {
                let trimmed = l.trim();
                if !trimmed.is_empty() {
                    files.push(repo_root.join(trimmed));
                }
            }
        }
    }

    // 3. git status --porcelain (untracked files and staged changes)
    if let Ok(out) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
    {
        if out.status.success() {
            for l in String::from_utf8_lossy(&out.stdout).lines() {
                if l.len() > 3 {
                    let path_str = l[3..].trim();
                    if !path_str.is_empty() {
                        files.push(repo_root.join(path_str));
                    }
                }
            }
        }
    }

    files
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

/// Return which of `projects` (owned) have files in the changed set.
pub fn filter_changed_projects(
    projects: Vec<crate::detect::Project>,
    changed: &[PathBuf],
) -> Vec<crate::detect::Project> {
    projects
        .into_iter()
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
