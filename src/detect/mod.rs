//! Core project detection types and trait.

use std::collections::HashMap;
use std::path::PathBuf;

pub mod docker;
pub mod dotnet;
pub mod go;
pub mod java;
pub mod make;
pub mod node;
pub mod python;
pub mod ruby;
pub mod rust;

/// The kind of project detected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectKind {
    Node,
    Python,
    Rust,
    Go,
    Make,
    Docker,
    Java,
    DotNet,
    Ruby,
}

impl std::fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProjectKind::Node => "node",
            ProjectKind::Python => "python",
            ProjectKind::Rust => "rust",
            ProjectKind::Go => "go",
            ProjectKind::Make => "make",
            ProjectKind::Docker => "docker",
            ProjectKind::Java => "java",
            ProjectKind::DotNet => "dotnet",
            ProjectKind::Ruby => "ruby",
        };
        f.write_str(s)
    }
}

/// A detected project.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Project {
    /// Short name, usually directory name.
    pub name: String,
    /// Root directory of the project.
    pub root: PathBuf,
    /// Detected project kind.
    pub kind: ProjectKind,
    /// Detected package manager (e.g. "npm", "poetry").
    pub package_manager: Option<String>,
    /// Scripts available in the project (e.g. from package.json or Makefile).
    pub scripts: HashMap<String, String>,
}

/// A resolved command ready to execute.
#[derive(Debug, Clone)]
pub struct CommandPlan {
    /// Executable program.
    pub program: String,
    /// Arguments to the program.
    pub args: Vec<String>,
    /// Working directory for the command.
    pub cwd: PathBuf,
    /// Environment overrides.
    pub env: HashMap<String, String>,
    /// Human-readable explanation of why this command was chosen.
    pub reason: String,
    /// Fallback command if the primary is unavailable (optional).
    pub fallback: Option<Box<CommandPlan>>,
    /// If Some, raw shell command from singularity.toml to be run through a shell.
    pub raw_shell: Option<String>,
}

impl CommandPlan {
    /// Build a simple plan with no env overrides and no fallback.
    pub fn simple(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
        cwd: PathBuf,
        reason: impl Into<String>,
    ) -> Self {
        CommandPlan {
            program: program.into(),
            args: args.into_iter().map(|a| a.into()).collect(),
            cwd,
            env: HashMap::new(),
            reason: reason.into(),
            fallback: None,
            raw_shell: None,
        }
    }

    /// Build a plan from a raw shell command string (e.g. from singularity.toml).
    pub fn shell(
        cmd: impl Into<String>,
        cwd: PathBuf,
        env: HashMap<String, String>,
        reason: impl Into<String>,
    ) -> Self {
        let raw = cmd.into();
        #[cfg(windows)]
        let (program, args) = ("cmd".to_string(), vec!["/C".to_string(), raw.clone()]);
        #[cfg(not(windows))]
        let (program, args) = ("sh".to_string(), vec!["-c".to_string(), raw.clone()]);

        CommandPlan {
            program,
            args,
            cwd,
            env,
            reason: reason.into(),
            fallback: None,
            raw_shell: Some(raw),
        }
    }

    /// Attach a fallback plan.
    pub fn with_fallback(mut self, fallback: CommandPlan) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }

    /// Append extra arguments to the plan, inserting npm's required "--" separator when appropriate.
    pub fn append_extra_args(&mut self, extra_args: &[String]) {
        if extra_args.is_empty() {
            return;
        }

        let needs_npm_dashdash = {
            let prog = self
                .program
                .trim_end_matches(".cmd")
                .trim_end_matches(".exe");
            let is_npm_prog = prog == "npm"
                && (self.args.first().map(|s| s.as_str()) == Some("run")
                    || self.args.first().map(|s| s.as_str()) == Some("test"));
            let is_npm_raw = if let Some(raw) = &self.raw_shell {
                let trimmed = raw.trim_start();
                trimmed.starts_with("npm run ") || trimmed.starts_with("npm test")
            } else {
                false
            };
            (is_npm_prog || is_npm_raw)
                && !self.args.iter().any(|a| a == "--")
                && extra_args.first().map(|s| s.as_str()) != Some("--")
        };

        if let Some(raw) = &mut self.raw_shell {
            raw.push(' ');
            if needs_npm_dashdash {
                raw.push_str("-- ");
            }
            raw.push_str(&extra_args.join(" "));
        }

        if needs_npm_dashdash {
            self.args.push("--".to_string());
        }

        self.args.extend(extra_args.iter().cloned());
    }
}

/// A verb — the unified command the user wants to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    Setup,
    Dev,
    Test,
    Build,
    Lint,
    Fmt,
    Run(String),
}

impl Verb {
    /// Parse from a string (used by --explain).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "setup" => Some(Verb::Setup),
            "dev" => Some(Verb::Dev),
            "test" => Some(Verb::Test),
            "build" => Some(Verb::Build),
            "lint" => Some(Verb::Lint),
            "fmt" => Some(Verb::Fmt),
            other => Some(Verb::Run(other.to_string())),
        }
    }

    /// Canonical name of this verb.
    pub fn name(&self) -> &str {
        match self {
            Verb::Setup => "setup",
            Verb::Dev => "dev",
            Verb::Test => "test",
            Verb::Build => "build",
            Verb::Lint => "lint",
            Verb::Fmt => "fmt",
            Verb::Run(_) => "run",
        }
    }
}

impl std::fmt::Display for Verb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verb::Run(s) => write!(f, "run {s}"),
            v => f.write_str(v.name()),
        }
    }
}

/// Trait implemented by each language/tool detector.
pub trait Detector: Send + Sync {
    /// Unique identifier for this detector.
    fn id(&self) -> &'static str;

    /// Try to detect a project rooted at `dir`. Returns `None` if not applicable.
    fn detect(&self, dir: &std::path::Path) -> Option<Project>;

    /// Produce a command plan for the given verb in the given project.
    fn plan(&self, project: &Project, verb: &Verb) -> Option<CommandPlan>;

    /// List of tools required by this project (name, version requirement).
    fn required_tools(&self, project: &Project) -> Vec<ToolRequirement>;
}

/// A tool that must be present for a project to work.
#[derive(Debug, Clone)]
pub struct ToolRequirement {
    /// Tool binary name.
    pub tool: String,
    /// Semver requirement string (e.g. ">=18.0.0"), empty if any version is fine.
    pub version_req: String,
    /// Install hint for the tool.
    pub install_hint: InstallHint,
}

/// How to install a missing tool.
#[derive(Debug, Clone)]
pub struct InstallHint {
    pub brew: Option<String>,
    pub apt: Option<String>,
    pub winget: Option<String>,
    pub url: Option<String>,
}

impl InstallHint {
    pub fn brew(pkg: impl Into<String>) -> Self {
        InstallHint {
            brew: Some(pkg.into()),
            apt: None,
            winget: None,
            url: None,
        }
    }
}

/// Return all built-in detectors in priority order.
pub fn all_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(node::NodeDetector),
        Box::new(python::PythonDetector),
        Box::new(rust::RustDetector),
        Box::new(go::GoDetector),
        Box::new(java::JavaDetector),
        Box::new(dotnet::DotNetDetector),
        Box::new(ruby::RubyDetector),
        Box::new(docker::DockerDetector),
        Box::new(make::MakeDetector),
    ]
}

/// Run all detectors against `dir` and return any matches.
pub fn detect_all(dir: &std::path::Path) -> Vec<Project> {
    all_detectors()
        .iter()
        .filter_map(|d| d.detect(dir))
        .collect()
}

/// Find the best single project for `dir` (first match wins in priority order).
pub fn detect_one(dir: &std::path::Path) -> Option<Project> {
    all_detectors().iter().find_map(|d| d.detect(dir))
}
