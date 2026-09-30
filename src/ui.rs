//! Terminal color and output utilities.

use owo_colors::OwoColorize;
use std::io::{self, IsTerminal};

/// Global output context.
pub struct Ui {
    pub color: bool,
    pub quiet: bool,
    pub verbose: bool,
}

impl Ui {
    pub fn new(no_color: bool, quiet: bool, verbose: bool) -> Self {
        let is_tty = io::stdout().is_terminal();
        let color_env_off = std::env::var("NO_COLOR").is_ok();
        let color = is_tty && !no_color && !color_env_off;
        Ui {
            color,
            quiet,
            verbose,
        }
    }

    /// Print a command being executed (cyan arrow prefix).
    pub fn print_cmd(&self, cmd: &str) {
        if self.quiet {
            return;
        }
        if self.color {
            eprintln!("  {}", format!("→ {cmd}").cyan());
        } else {
            eprintln!("  → {cmd}");
        }
    }

    /// Print a success message.
    pub fn success(&self, msg: &str) {
        if self.quiet {
            return;
        }
        if self.color {
            eprintln!("{}", msg.green());
        } else {
            eprintln!("{msg}");
        }
    }

    /// Print an error message.
    pub fn error(&self, msg: &str) {
        if self.color {
            eprintln!("{}", msg.red());
        } else {
            eprintln!("{msg}");
        }
    }

    /// Print a warning message.
    pub fn warn(&self, msg: &str) {
        if self.quiet {
            return;
        }
        if self.color {
            eprintln!("{}", msg.yellow());
        } else {
            eprintln!("{msg}");
        }
    }

    /// Print a verbose message (only when --verbose).
    pub fn verbose(&self, msg: &str) {
        if self.verbose {
            eprintln!("{msg}");
        }
    }
}
