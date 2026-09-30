//! ASCII banner for sgl.
//!
//! Prints a cyan-to-magenta gradient on truecolor terminals,
//! plain cyan on 16-color terminals, and no color otherwise.
//! Does not print when stdout is not a TTY or when quiet is set.

use std::io::{self, IsTerminal};

use owo_colors::OwoColorize;

/// The raw ASCII art lines (no color).
const BANNER_LINES: &[&str] = &[
    r"     _                   _            _ _",
    r" ___(_)_ __   __ _ _   _| | __ _ _ __(_) |_ _   _",
    r"/ __| | '_ \ / _` | | | | |/ _` | '__| | __| | | |",
    r"\__ \ | | | | (_| | |_| | | (_| | |  | | |_| |_| |",
    r"|___/_|_| |_|\__, |\__,_|_|\__,_|_|  |_|\__|\__, |",
    r"             |___/                          |___/",
];

/// Print the banner to stdout. Respects TTY check and `--quiet`.
pub fn print_banner(quiet: bool) {
    if quiet || !io::stdout().is_terminal() {
        return;
    }

    let truecolor = supports_truecolor();
    let color16 = supports_16color();

    for (i, line) in BANNER_LINES.iter().enumerate() {
        if truecolor {
            // Gradient from cyan (#00d4ff) to magenta (#c040c0).
            let t = i as f32 / (BANNER_LINES.len() - 1) as f32;
            let r = lerp(0, 192, t);
            let g = lerp(212, 64, t);
            let b = lerp(255, 192, t);
            println!("{}", line.truecolor(r, g, b));
        } else if color16 {
            println!("{}", line.cyan());
        } else {
            println!("{line}");
        }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn supports_truecolor() -> bool {
    if let Ok(ct) = std::env::var("COLORTERM") {
        return ct.eq_ignore_ascii_case("truecolor") || ct.eq_ignore_ascii_case("24bit");
    }
    false
}

fn supports_16color() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    std::env::var("TERM").map(|t| t != "dumb").unwrap_or(false)
}
