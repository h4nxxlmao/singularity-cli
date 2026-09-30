//! Command execution: spawn child processes, stream I/O, forward signals.

use anyhow::Result;
use std::process::Stdio;

use crate::detect::CommandPlan;
use crate::ui::Ui;
use owo_colors::OwoColorize;

/// Execute a command plan, streaming stdout/stderr and passing through stdin.
/// Returns the child's exit code.
pub async fn run(plan: &CommandPlan, ui: &Ui, dry_run: bool) -> Result<i32> {
    let cmd_display = format_cmd(plan);
    ui.print_cmd(&cmd_display);

    if dry_run {
        return Ok(0);
    }

    // Check if the program exists; if not, try fallback.
    if !program_exists(&plan.program) {
        if let Some(fallback) = &plan.fallback {
            ui.warn(&format!("  '{}' not found, trying fallback", plan.program,));
            return Box::pin(run(fallback, ui, dry_run)).await;
        }
        ui.error(&format!(
            "  '{}' not found. Install it first. Exit 127.",
            plan.program
        ));
        return Ok(127);
    }

    let mut cmd = build_command(plan);
    let status = cmd.status().await?;

    let code = status.code().unwrap_or(1);
    Ok(code)
}

/// Execute multiple plans sequentially (for singularity.toml list commands).
pub async fn run_sequence(plans: &[CommandPlan], ui: &Ui, dry_run: bool) -> Result<i32> {
    for plan in plans {
        let code = run(plan, ui, dry_run).await?;
        if code != 0 {
            return Ok(code);
        }
    }
    Ok(0)
}

fn format_cmd(plan: &CommandPlan) -> String {
    if plan.args.is_empty() {
        plan.program.clone()
    } else {
        format!("{} {}", plan.program, plan.args.join(" "))
    }
}

fn program_exists(program: &str) -> bool {
    which::which(program).is_ok()
}

fn build_command(plan: &CommandPlan) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(&plan.program);
    cmd.args(&plan.args);
    cmd.current_dir(&plan.cwd);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    for (k, v) in &plan.env {
        cmd.env(k, v);
    }

    cmd
}

/// Print plan details for --explain mode (does not run).
pub fn explain_plan(plan: &CommandPlan, reason: &str, ui: &Ui) {
    let cmd_str = format_cmd(plan);
    if ui.color {
        eprintln!("  {}", format!("→ {cmd_str}").cyan());
    } else {
        eprintln!("  → {cmd_str}");
    }
    eprintln!("     cwd:    {}", plan.cwd.display());
    if !plan.env.is_empty() {
        let env_display: Vec<String> = plan.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        eprintln!("     env:    {}", env_display.join(", "));
    }
    eprintln!("     reason: {reason}");
}
