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
                    files.push(crate::normalize_path(&repo_root.join(trimmed)));
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
                    files.push(crate::normalize_path(&repo_root.join(trimmed)));
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
                        files.push(crate::normalize_path(&repo_root.join(path_str)));
                    }
                }
            }
        }
    }

    files
}

pub fn matches_project(file_path: &Path, project_root: &Path, repo_root: &Path) -> bool {
    let f_norm = crate::normalize_path(file_path);
    let p_norm = crate::normalize_path(project_root);
    if f_norm.starts_with(&p_norm) {
        return true;
    }
    if let (Ok(f_can), Ok(p_can)) = (file_path.canonicalize(), project_root.canonicalize()) {
        if f_can.starts_with(&p_can) {
            return true;
        }
    }
    let p_rel_str = project_root
        .strip_prefix(repo_root)
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| project_root.to_string_lossy().replace('\\', "/"));
    let f_str = file_path.to_string_lossy().replace('\\', "/");
    let target = format!("/{p_rel_str}/");
    if f_str.contains(&target)
        || f_str.ends_with(&format!("/{p_rel_str}"))
        || f_str.starts_with(&format!("{p_rel_str}/"))
    {
        return true;
    }
    false
}

/// Return which of `projects` have files in the changed set.
pub fn filter_changed<'a>(
    projects: &'a [crate::detect::Project],
    changed: &[PathBuf],
    repo_root: &Path,
) -> Vec<&'a crate::detect::Project> {
    projects
        .iter()
        .filter(|p| {
            changed
                .iter()
                .any(|f| matches_project(f, &p.root, repo_root))
        })
        .collect()
}

/// Return which of `projects` (owned) have files in the changed set.
pub fn filter_changed_projects(
    projects: Vec<crate::detect::Project>,
    changed: &[PathBuf],
    repo_root: &Path,
) -> Vec<crate::detect::Project> {
    projects
        .into_iter()
        .filter(|p| {
            changed
                .iter()
                .any(|f| matches_project(f, &p.root, repo_root))
        })
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
        Some(crate::normalize_path(Path::new(s.trim())))
    } else {
        None
    }
}
