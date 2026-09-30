//! Terminal color and output utilities.

use owo_colors::OwoColorize;
use std::io::{self, IsTerminal};

/// Global output context.
pub struct Ui {
    pub color: bool,
    pub quiet: bool,
    pub verbose: bool,
    /// Whether the terminal can render Unicode arrows (→). Falls back to "->".
    pub unicode: bool,
}

impl Ui {
    pub fn new(no_color: bool, quiet: bool, verbose: bool) -> Self {
        let is_tty = io::stdout().is_terminal();
        let color_env_off = std::env::var("NO_COLOR").is_ok();
        let color = is_tty && !no_color && !color_env_off;

        // Unicode arrows are safe when color is enabled (implies UTF-8-capable tty)
        // and the console code page is UTF-8 on Windows.
        let unicode = color && is_utf8_console();

        Ui {
            color,
            quiet,
            verbose,
            unicode,
        }
    }

    /// Return the arrow prefix: "→" when Unicode is available, "->" otherwise.
    pub fn arrow(&self) -> &'static str {
        if self.unicode {
            "→"
        } else {
            "->"
        }
    }

    /// Print a command being executed (arrow prefix on stderr).
    pub fn print_cmd(&self, cmd: &str) {
        if self.quiet {
            return;
        }
        let arrow = self.arrow();
        if self.color {
            eprintln!("  {}", format!("{arrow} {cmd}").cyan());
        } else {
            eprintln!("  {arrow} {cmd}");
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

/// Returns true when stdout can render UTF-8 characters reliably.
///
/// On Windows, checks that the active code page is 65001 (UTF-8).
/// On Unix, always true (locales default to UTF-8 in modern systems).
fn is_utf8_console() -> bool {
    #[cfg(windows)]
    {
        // GetConsoleOutputCP() returns 65001 when the console is set to UTF-8.
        extern "system" {
            fn GetConsoleOutputCP() -> u32;
        }
        let cp = unsafe { GetConsoleOutputCP() };
        cp == 65001
    }
    #[cfg(not(windows))]
    {
        true
    }
}
