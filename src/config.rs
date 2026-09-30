//! `singularity.toml` configuration loading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::detect::{CommandPlan, Verb};

/// Top-level structure of singularity.toml.
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub project: ProjectConfig,
    pub commands: HashMap<String, CommandValue>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ProjectConfig {
    pub name: Option<String>,
}

/// A command value can be a single string or a list of strings.
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum CommandValue {
    Single(String),
    List(Vec<String>),
}

impl CommandValue {
    /// Expand to a list of shell command strings.
    pub fn commands(&self) -> Vec<&str> {
        match self {
            CommandValue::Single(s) => vec![s.as_str()],
            CommandValue::List(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

/// Load config from `dir/singularity.toml`. Returns `None` if the file doesn't exist.
pub fn load(dir: &Path) -> Result<Option<(Config, PathBuf)>> {
    let path = dir.join("singularity.toml");
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some((config, path)))
}

/// Resolve a verb override from config. Returns command plans if an override exists.
pub fn override_for_verb(config: &Config, verb: &Verb, cwd: &Path) -> Option<Vec<CommandPlan>> {
    let key = match verb {
        Verb::Run(s) => s.as_str(),
        v => v.name(),
    };

    let value = config.commands.get(key)?;
    let plans = value
        .commands()
        .iter()
        .map(|cmd| shell_plan(cmd, cwd, config, "singularity.toml override"))
        .collect();
    Some(plans)
}

/// Check if a verb override exists (for --explain).
pub fn has_override(config: &Config, verb: &Verb) -> bool {
    let key = match verb {
        Verb::Run(s) => s.as_str(),
        v => v.name(),
    };
    config.commands.contains_key(key)
}

/// Build a CommandPlan from a shell command string and the config env.
fn shell_plan(cmd: &str, cwd: &Path, config: &Config, reason: &str) -> CommandPlan {
    // Split into program + args (simple whitespace split; no shell expansion)
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap_or("sh").to_string();
    let args: Vec<String> = parts.map(|s| s.to_string()).collect();

    CommandPlan {
        program,
        args,
        cwd: cwd.to_path_buf(),
        env: config.env.clone(),
        reason: reason.to_string(),
        fallback: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn loads_config() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("singularity.toml"),
            r#"
[project]
name = "api"

[commands]
test = "pytest -x"
dev = ["docker compose up -d db", "uvicorn app:app --reload"]

[env]
DATABASE_URL = "postgres://localhost/dev"
"#,
        )
        .unwrap();

        let (config, _) = load(dir).unwrap().unwrap();
        assert_eq!(config.project.name.as_deref(), Some("api"));
        assert!(config.commands.contains_key("test"));
        assert_eq!(
            config.env.get("DATABASE_URL").map(|s| s.as_str()),
            Some("postgres://localhost/dev")
        );
    }

    #[test]
    fn no_config_returns_none() {
        let tmp = TempDir::new().unwrap();
        let result = load(tmp.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn override_single_command() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("singularity.toml"),
            "[commands]\ntest = \"pytest -x\"\n",
        )
        .unwrap();
        let (config, _) = load(dir).unwrap().unwrap();
        let plans = override_for_verb(&config, &Verb::Test, dir).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].program, "pytest");
        assert!(plans[0].args.contains(&"-x".to_string()));
    }

    #[test]
    fn override_list_command() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();
        fs::write(
            dir.join("singularity.toml"),
            r#"[commands]
dev = ["docker compose up -d db", "uvicorn app:app --reload"]
"#,
        )
        .unwrap();
        let (config, _) = load(dir).unwrap().unwrap();
        let plans = override_for_verb(&config, &Verb::Dev, dir).unwrap();
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].program, "docker");
        assert_eq!(plans[1].program, "uvicorn");
    }
}
