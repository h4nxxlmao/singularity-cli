//! CLI argument definitions using clap derive.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "sgl",
    version,
    about = "One set of commands for every project type. https://getsingularity.lol/cli",
    long_about = None,
    disable_help_subcommand = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Print the commands that would run without executing them.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Show extra output.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Suppress non-error output.
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Disable color output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Run as if in this directory.
    #[arg(long, global = true, value_name = "PATH")]
    pub cwd: Option<PathBuf>,

    /// Limit to this project name (monorepo).
    #[arg(long, global = true, value_name = "NAME")]
    pub project: Option<String>,

    /// Only run in projects with files changed against base branch (monorepo).
    #[arg(long, global = true)]
    pub changed: bool,

    /// Run monorepo projects sequentially instead of in parallel.
    #[arg(long, global = true)]
    pub serial: bool,

    /// Abort execution on the first project failure (monorepo).
    #[arg(long, global = true)]
    pub fail_fast: bool,

    /// Output as JSON (for info, doctor, and --explain).
    #[arg(long, global = true)]
    pub json: bool,

    /// Print the commands for VERB without running them.
    #[arg(long, global = true, value_name = "VERB")]
    pub explain: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install dependencies.
    Setup,
    /// Start the dev server or watcher.
    Dev,
    /// Run tests.
    Test {
        /// Extra args forwarded to the test runner.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build the project.
    Build,
    /// Lint and format check.
    Lint,
    /// Auto-format source files.
    Fmt,
    /// Run a project script or task.
    Run {
        /// Script or target name.
        script: String,
        /// Extra args forwarded to the script.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Check for missing tools and version mismatches.
    Doctor,
    /// Show detected projects, tools, and resolved commands.
    Info,
    /// Write a singularity.toml based on what was detected.
    Init,
    /// Print shell completions.
    Completions {
        /// Shell name: bash, zsh, fish, or powershell.
        shell: String,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
