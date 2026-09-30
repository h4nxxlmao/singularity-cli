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

use std::path::{Path, PathBuf};
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

pub fn normalize_path(p: &Path) -> PathBuf {
    let abs = if p.is_relative() {
        if let Ok(cur) = std::env::current_dir() {
            cur.join(p)
        } else {
            p.to_path_buf()
        }
    } else {
        p.to_path_buf()
    };

    #[cfg(windows)]
    {
        let s = abs.to_string_lossy().replace('/', "\\");
        let clean = s.strip_prefix(r"\\?\").unwrap_or(&s);
        PathBuf::from(clean)
    }
    #[cfg(not(windows))]
    {
        abs
    }
}

async fn run(args: Cli) -> Result<()> {
    let ui = Ui::new(args.no_color, args.quiet, args.verbose);

    // Resolve working directory
    let cwd: PathBuf = if let Some(p) = &args.cwd {
        normalize_path(p)
    } else {
        normalize_path(&std::env::current_dir()?)
    };

    // Load singularity.toml if present
    let config_result = config::load(&cwd)?;
    let cfg = config_result.as_ref().map(|(c, _)| c);

    // Handle --explain <verb>
    if let Some(verb_str) = &args.explain {
        let verb = Verb::parse(verb_str).unwrap_or_else(|| Verb::Run(verb_str.clone()));
        let explanations = resolve::explain(&cwd, &verb, cfg);
        if explanations.is_empty() {
            let detected = resolve::detect_one_in(&cwd);
            let msg = resolve::format_unavailable_message(&cwd, verb_str, detected.as_ref());
            if args.json {
                let out = serde_json::json!({
                    "schema_version": 1,
                    "verb": verb_str,
                    "error": msg,
                    "plans": []
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                ui.error(&msg);
            }
            process::exit(1);
        }
        if args.json {
            let plans_json: Vec<serde_json::Value> = explanations
                .iter()
                .map(|(reason, plan)| {
                    serde_json::json!({
                        "program": plan.program,
                        "args": plan.args,
                        "cwd": plan.cwd,
                        "env": plan.env,
                        "reason": reason,
                    })
                })
                .collect();
            let out = serde_json::json!({
                "schema_version": 1,
                "verb": verb_str,
                "plans": plans_json,
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            return Ok(());
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
        if !args.quiet && !args.json {
            print_verb_list();
        }
        return Ok(());
    }

    let exit_code = match args.command.as_ref().unwrap() {
        Commands::Setup => dispatch_verb(&cwd, Verb::Setup, vec![], &args, cfg, &ui).await?,
        Commands::Dev => dispatch_verb(&cwd, Verb::Dev, vec![], &args, cfg, &ui).await?,
        Commands::Test { args: extra } => {
            dispatch_verb(&cwd, Verb::Test, extra.clone(), &args, cfg, &ui).await?
        }
        Commands::Build => dispatch_verb(&cwd, Verb::Build, vec![], &args, cfg, &ui).await?,
        Commands::Lint => dispatch_verb(&cwd, Verb::Lint, vec![], &args, cfg, &ui).await?,
        Commands::Fmt => dispatch_verb(&cwd, Verb::Fmt, vec![], &args, cfg, &ui).await?,
        Commands::Run {
            script,
            args: extra,
        } => {
            dispatch_verb(
                &cwd,
                Verb::Run(script.clone()),
                extra.clone(),
                &args,
                cfg,
                &ui,
            )
            .await?
        }
        Commands::Doctor => doctor::run(&cwd, &ui, args.json, args.project.as_deref()),
        Commands::Info => {
            print_info(&cwd, cfg, &ui, args.json);
            0
        }
        Commands::Init => {
            init_config(&cwd, cfg, &ui, args.dry_run)?;
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

async fn dispatch_verb(
    cwd: &Path,
    verb: Verb,
    extra_args: Vec<String>,
    args: &Cli,
    cfg: Option<&config::Config>,
    ui: &Ui,
) -> Result<i32> {
    let is_monorepo = monorepo::is_monorepo(cwd) || args.changed;
    if is_monorepo {
        let opts = monorepo::MonorepoOptions {
            verb,
            extra_args,
            project_filter: args.project.clone(),
            changed: args.changed,
            serial: args.serial,
            fail_fast: args.fail_fast,
            dry_run: args.dry_run,
        };
        monorepo::run_monorepo(cwd, opts, ui, cfg).await
    } else {
        run_verb_with_args(
            cwd,
            verb,
            &extra_args,
            args.project.as_deref(),
            cfg,
            ui,
            args.dry_run,
        )
        .await
    }
}

async fn run_verb_with_args(
    cwd: &Path,
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
                    last.append_extra_args(extra_args);
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
            let e_str = e.to_string();
            if e_str.starts_with("sgl: ") {
                ui.error(&e_str);
            } else {
                ui.error(&format!("sgl: {e_str}"));
            }
            Ok(1)
        }
    }
}

fn print_info(cwd: &Path, cfg: Option<&config::Config>, _ui: &Ui, json: bool) {
    let subs = monorepo::discover(cwd);
    let mut projects = if subs.len() > 1 || (!subs.is_empty() && monorepo::is_workspace_root(cwd)) {
        subs
    } else {
        let top = detect_all(cwd);
        if top.is_empty() {
            subs
        } else {
            top
        }
    };
    if projects.len() > 1 {
        let top = detect_all(cwd);
        if !top.is_empty() {
            projects.insert(0, top[0].clone());
        }
    }

    if json {
        let val = serde_json::json!({
            "schema_version": 1,
            "directory": cwd,
            "projects": projects,
            "has_config": cfg.is_some(),
        });
        println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
        return;
    }

    if projects.is_empty() {
        eprintln!("no projects detected in this directory.");
        return;
    }

    if projects.len() > 1 {
        // Tree display for monorepo
        let root_name = dir_name(cwd);
        eprintln!("{root_name}");
        for (i, p) in projects.iter().enumerate() {
            let is_last = i == projects.len() - 1;
            let branch = if is_last { "└── " } else { "├── " };
            let pm = p
                .package_manager
                .as_deref()
                .map(|pm| format!(" ({pm})"))
                .unwrap_or_default();
            let rel = p
                .root
                .strip_prefix(cwd)
                .map(|r| r.display().to_string())
                .unwrap_or_else(|_| p.name.clone());
            eprintln!("{branch}{} [{kind}]{pm} ({rel})", p.name, kind = p.kind);
        }
    } else {
        for p in &projects {
            let pm = p
                .package_manager
                .as_deref()
                .map(|pm| format!(" ({pm})"))
                .unwrap_or_default();
            eprintln!("  {} [{}]{}", p.name, p.kind, pm);
        }
    }

    if let Some(cfg) = cfg {
        if let Some(name) = &cfg.project.name {
            eprintln!("  config: singularity.toml (project: {name})");
        } else {
            eprintln!("  config: singularity.toml");
        }
    }
}

fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

fn print_verb_list() {
    eprintln!("\nAvailable commands:");
    for (verb, desc) in VERB_LIST {
        eprintln!("  sgl {verb:<12} {desc}");
    }
    eprintln!();
    eprintln!(
        "Global flags: --dry-run  --explain <verb>  --json  --quiet  --verbose  --cwd <path>  --project <name>  --changed  --serial  --fail-fast"
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

fn init_config(cwd: &Path, _cfg: Option<&config::Config>, ui: &Ui, dry_run: bool) -> Result<()> {
    let projects = detect_all(cwd);
    if projects.is_empty() {
        ui.warn("no projects detected in this directory; nothing to write");
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

    let content = lines.join("\n");
    if dry_run {
        println!("{}", dest.display());
        println!("{}", content);
    } else {
        std::fs::write(&dest, &content)?;
        println!("{}", content);
        ui.success(&format!("wrote {}", dest.display()));
    }
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
