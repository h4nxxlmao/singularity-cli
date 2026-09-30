//! sgl — universal project command runner.
//!
//! Entry point: parse args, resolve working directory, load config if present,
//! dispatch to the appropriate handler.

pub mod banner;
pub mod cli;
pub mod config;
pub mod detect;
pub mod doctor;
pub mod exec;
pub mod git;
pub mod monorepo;
pub mod resolve;
pub mod ui;

use std::path::PathBuf;
use std::process;

use anyhow::Result;

use cli::{Cli, Commands};
use detect::{detect_all, Verb};
use ui::Ui;

#[tokio::main]
async fn main() {
    let args = cli::parse();
    if let Err(e) = run(args).await {
        eprintln!("sgl: {e:#}");
        process::exit(1);
    }
}

async fn run(args: Cli) -> Result<()> {
    let ui = Ui::new(args.no_color, args.quiet, args.verbose);

    // Resolve working directory
    let cwd: PathBuf = if let Some(p) = &args.cwd {
        p.clone()
    } else {
        std::env::current_dir()?
    };

    // Load singularity.toml if present
    let config_result = config::load(&cwd)?;
    let cfg = config_result.as_ref().map(|(c, _)| c);

    // Handle --explain <verb>
    if let Some(verb_str) = &args.explain {
        let verb = Verb::parse(verb_str).unwrap_or_else(|| Verb::Run(verb_str.clone()));
        let explanations = resolve::explain(&cwd, &verb, cfg);
        if explanations.is_empty() {
            ui.error(&format!("no command found for '{verb_str}'"));
            process::exit(1);
        }
        for (reason, plan) in &explanations {
            exec::explain_plan(plan, reason, &ui);
        }
        return Ok(());
    }

    // No subcommand: print banner + info + verb list
    if args.command.is_none() {
        banner::print_banner(args.quiet);
        print_info(&cwd, cfg, &ui, args.json);
        if !args.quiet {
            print_verb_list();
        }
        return Ok(());
    }

    let exit_code = match args.command.as_ref().unwrap() {
        Commands::Setup => {
            run_verb(
                &cwd,
                Verb::Setup,
                args.project.as_deref(),
                cfg,
                &ui,
                args.dry_run,
            )
            .await?
        }
        Commands::Dev => {
            run_verb(
                &cwd,
                Verb::Dev,
                args.project.as_deref(),
                cfg,
                &ui,
                args.dry_run,
            )
            .await?
        }
        Commands::Test { args: extra } => {
            run_verb_with_args(
                &cwd,
                Verb::Test,
                extra,
                args.project.as_deref(),
                cfg,
                &ui,
                args.dry_run,
            )
            .await?
        }
        Commands::Build => {
            run_verb(
                &cwd,
                Verb::Build,
                args.project.as_deref(),
                cfg,
                &ui,
                args.dry_run,
            )
            .await?
        }
        Commands::Lint => {
            run_verb(
                &cwd,
                Verb::Lint,
                args.project.as_deref(),
                cfg,
                &ui,
                args.dry_run,
            )
            .await?
        }
        Commands::Fmt => {
            run_verb(
                &cwd,
                Verb::Fmt,
                args.project.as_deref(),
                cfg,
                &ui,
                args.dry_run,
            )
            .await?
        }
        Commands::Run { script, .. } => {
            run_verb(
                &cwd,
                Verb::Run(script.clone()),
                args.project.as_deref(),
                cfg,
                &ui,
                args.dry_run,
            )
            .await?
        }
        Commands::Doctor => doctor::run(&cwd, &ui, args.json),
        Commands::Info => {
            print_info(&cwd, cfg, &ui, args.json);
            0
        }
        Commands::Init => {
            init_config(&cwd, cfg, &ui)?;
            0
        }
        Commands::Completions { shell } => {
            print_completions(shell);
            0
        }
    };

    if exit_code != 0 {
        process::exit(exit_code);
    }

    Ok(())
}

async fn run_verb(
    cwd: &std::path::Path,
    verb: Verb,
    project_filter: Option<&str>,
    cfg: Option<&config::Config>,
    ui: &Ui,
    dry_run: bool,
) -> Result<i32> {
    match resolve::resolve(cwd, &verb, project_filter, cfg) {
        Ok(resolutions) => {
            let mut code = 0;
            for res in resolutions {
                let c = exec::run_sequence(&res.plans, ui, dry_run).await?;
                if c != 0 {
                    code = c;
                    break;
                }
            }
            Ok(code)
        }
        Err(e) => {
            ui.error(&format!("sgl: {e}"));
            Ok(1)
        }
    }
}

async fn run_verb_with_args(
    cwd: &std::path::Path,
    verb: Verb,
    extra_args: &[String],
    project_filter: Option<&str>,
    cfg: Option<&config::Config>,
    ui: &Ui,
    dry_run: bool,
) -> Result<i32> {
    match resolve::resolve(cwd, &verb, project_filter, cfg) {
        Ok(resolutions) => {
            let mut code = 0;
            for mut res in resolutions {
                // Append extra args to the last plan
                if let Some(last) = res.plans.last_mut() {
                    last.args.extend(extra_args.iter().cloned());
                }
                let c = exec::run_sequence(&res.plans, ui, dry_run).await?;
                if c != 0 {
                    code = c;
                    break;
                }
            }
            Ok(code)
        }
        Err(e) => {
            ui.error(&format!("sgl: {e}"));
            Ok(1)
        }
    }
}

fn print_info(cwd: &std::path::Path, cfg: Option<&config::Config>, ui: &Ui, json: bool) {
    let mut projects = detect_all(cwd);

    // If nothing detected at top level, look for monorepo sub-projects.
    if projects.is_empty() {
        projects = monorepo::discover(cwd);
    }

    if json {
        let val = serde_json::json!({
            "directory": cwd,
            "projects": projects,
            "has_config": cfg.is_some(),
        });
        println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
        return;
    }

    if projects.is_empty() {
        ui.warn("No projects detected in this directory.");
        return;
    }

    if !ui.quiet {
        for p in &projects {
            let pm = p
                .package_manager
                .as_deref()
                .map(|pm| format!(" ({pm})"))
                .unwrap_or_default();
            eprintln!("  {} [{}]{}", p.name, p.kind, pm);
        }
        if let Some(cfg) = cfg {
            if let Some(name) = &cfg.project.name {
                eprintln!("  config: singularity.toml (project: {name})");
            } else {
                eprintln!("  config: singularity.toml");
            }
        }
    }
}

fn print_verb_list() {
    eprintln!("\nAvailable commands:");
    for (verb, desc) in VERB_LIST {
        eprintln!("  sgl {verb:<12} {desc}");
    }
    eprintln!();
    eprintln!(
        "Global flags: --dry-run  --explain <verb>  --json  --quiet  --verbose  --cwd <path>"
    );
}

const VERB_LIST: &[(&str, &str)] = &[
    ("setup", "install dependencies"),
    ("dev", "start the dev server or watcher"),
    ("test [args]", "run tests"),
    ("build", "build"),
    ("lint", "lint and format check"),
    ("fmt", "auto-format"),
    ("run <script>", "run a project script or task"),
    ("doctor", "check for missing tools and version mismatches"),
    ("info", "show detected projects and resolved commands"),
    ("init", "write singularity.toml from detected config"),
    ("completions <shell>", "print shell completions"),
];

fn init_config(cwd: &std::path::Path, _cfg: Option<&config::Config>, ui: &Ui) -> Result<()> {
    let projects = detect_all(cwd);
    if projects.is_empty() {
        ui.warn("no projects detected; nothing to write");
        return Ok(());
    }

    let dest = cwd.join("singularity.toml");
    if dest.exists() {
        ui.warn("singularity.toml already exists; not overwriting");
        return Ok(());
    }

    let mut lines = Vec::new();
    if let Some(first) = projects.first() {
        lines.push("[project]".to_string());
        lines.push(format!("name = \"{}\"", first.name));
        lines.push(String::new());
    }
    lines.push("[commands]".to_string());
    lines.push("# override any verb here, e.g.:".to_string());
    lines.push("# test = \"pytest -x\"".to_string());
    lines.push(String::new());
    lines.push("[env]".to_string());
    lines.push("# environment variables for all commands".to_string());

    std::fs::write(&dest, lines.join("\n"))?;
    ui.success(&format!("wrote {}", dest.display()));
    Ok(())
}

fn print_completions(shell: &str) {
    match shell.to_lowercase().as_str() {
        "bash" => println!("{BASH_COMPLETIONS}"),
        "zsh" => println!("{ZSH_COMPLETIONS}"),
        "fish" => println!("{FISH_COMPLETIONS}"),
        "powershell" | "pwsh" => println!("{POWERSHELL_COMPLETIONS}"),
        other => eprintln!("unknown shell '{other}'; choose: bash, zsh, fish, powershell"),
    }
}

const BASH_COMPLETIONS: &str = r#"
_sgl_completions() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local verbs="setup dev test build lint fmt run doctor info init completions"
  COMPREPLY=($(compgen -W "$verbs" -- "$cur"))
}
complete -F _sgl_completions sgl
"#;

const ZSH_COMPLETIONS: &str = r#"
#compdef sgl
_sgl() {
  local -a verbs
  verbs=(setup dev test build lint fmt run doctor info init completions)
  _describe 'command' verbs
}
compdef _sgl sgl
"#;

const FISH_COMPLETIONS: &str = r#"
complete -c sgl -f -n '__fish_use_subcommand' -a setup -d 'install dependencies'
complete -c sgl -f -n '__fish_use_subcommand' -a dev -d 'start dev server'
complete -c sgl -f -n '__fish_use_subcommand' -a test -d 'run tests'
complete -c sgl -f -n '__fish_use_subcommand' -a build -d 'build'
complete -c sgl -f -n '__fish_use_subcommand' -a lint -d 'lint'
complete -c sgl -f -n '__fish_use_subcommand' -a fmt -d 'auto-format'
complete -c sgl -f -n '__fish_use_subcommand' -a run -d 'run a script'
complete -c sgl -f -n '__fish_use_subcommand' -a doctor -d 'check tools'
complete -c sgl -f -n '__fish_use_subcommand' -a info -d 'show info'
complete -c sgl -f -n '__fish_use_subcommand' -a init -d 'write singularity.toml'
complete -c sgl -f -n '__fish_use_subcommand' -a completions -d 'shell completions'
"#;

const POWERSHELL_COMPLETIONS: &str = r#"
Register-ArgumentCompleter -Native -CommandName sgl -ScriptBlock {
  param($wordToComplete, $commandAst, $cursorPosition)
  $verbs = @('setup','dev','test','build','lint','fmt','run','doctor','info','init','completions')
  $verbs | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
    [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
  }
}
"#;
