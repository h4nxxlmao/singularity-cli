# NOTES.md — singularity-cli decisions and status

## Architecture & Specifications

- **Binary & Package Name**: The binary is `sgl`. The package name published on crates.io is `singularitycli` (`singularity-cli` was taken by an unrelated project).
- **Streams**: `--explain` prints to stderr (same stream as `print_cmd`). Machine-readable JSON output (`--json`) is printed strictly to stdout.
- **Config Syntax Errors**: `config::load()` captures TOML parse errors with exact file, line, and column coordinates (`singularity.toml:line:col: message`) using `toml::de::Error::span()`.
- **Precedence**: `singularity.toml` overrides detected project commands. When commands are specified under `[commands]`, those exact commands take precedence over any heuristic detection.
- **Signals & Stdin**: Children inherit stdin, stdout, and stderr for interactive commands (e.g. `sgl dev`, REPLs, interactive tests). Signals (Ctrl+C / SIGINT) are intercepted by Tokio signal handlers and propagate termination to the running child processes.
- **Windows Command Resolution**: Resolves relative executable paths (`./script.sh`, `bin/tool`) and checks `PATHEXT` on Windows (`.exe`, `.cmd`, `.bat`) so tools installed via npm/pip (`npm.cmd`, `pytest.exe`) are located and executed directly without cmd invocation issues.
- **Missing Tool Exit Code**: When a resolved binary is missing from PATH, `sgl` exits with code `127` and displays platform-specific installation hints (`brew`, `apt`, `winget`).
- **Exit Code Passthrough**: When a child process terminates with a non-zero exit code (e.g., code 3), `sgl` passes that exact exit code through.
- **Unsupported Verbs**: When an unsupported verb is requested, `sgl` exits with code 1. For Node projects, it inspects `package.json` and lists all defined npm scripts. For other project types, it lists the available standard verbs.

## Verb Mappings (Strict Mode)

- **Never Guess Tools**: Default guessing of test or lint runners (such as auto-running `npx jest` or `npx eslint` when not defined in `package.json`) has been eliminated. If a Node project has no matching script, `sgl` reports `no <verb> script in package.json` and lists available scripts.
- **Ruby**:
  - `lint`: `bundle exec rubocop`
  - `test`: `bundle exec rspec` if `spec/` directory exists; otherwise `bundle exec rake test`.
- **Java (Maven & Gradle)**:
  - Strict linting: Only generates a lint command if `checkstyle`, `spotless`, or `ktlint` plugins are explicitly configured in `pom.xml` / `build.gradle` / `build.gradle.kts`.
- **Docker**:
  - Only maps `setup` (`docker compose pull`), `dev` (`docker compose up`), and `build` (`docker compose build`). `lint`, `fmt`, and `test` return `None`.
- **Rust**:
  - Automatically detects `[workspace]` in `Cargo.toml`, applying `--workspace` to clippy and `--all` to fmt.
- **Go**:
  - Only attaches Makefile targets under `sgl run <target>` rather than executing them as `go run <target>`.
- **Makefile Merging**:
  - Language-detected projects (Node, Rust, Go, Python, etc.) that also contain a `Makefile` merge make targets into `sgl run <target>`, allowing `sgl run <custom_target>` to execute Makefile recipes seamlessly.

## Monorepo Execution

- **Parallel Runner**: Executes verbs across sub-projects concurrently, bounded by `num_cpus::get()`.
- **Serial Mode**: `--serial` sets concurrency to 1, executing sub-projects sequentially.
- **Fail-Fast**: `--fail-fast` halts execution immediately upon the first project failure.
- **Changed-Only**: `--changed` filters sub-projects based on Git modifications (staged, unstaged, and untracked files compared against HEAD).
- **Prefix Streaming**: For non-dev verbs running in parallel, project output lines are buffered and streamed prefixed with `[<project>]`.
- **Dev Mode**: `sgl dev` runs all workspace dev servers concurrently until interrupted by Ctrl+C.
- **Summary Table**: Emits a formatted summary table displaying `Project`, `Verb`, `Status` (✓ or ✗), and `Duration`. Exits non-zero if any project fails.

## Machine-Readable JSON Output

All JSON outputs include `"schema_version": 1`:
- `sgl info --json`: outputs detected project kind, runner, resolved verbs, root directory, and monorepo packages.
- `sgl doctor --json`: outputs structured diagnostic checks with tool names, installed versions, required versions, status, and fix suggestions.
- `sgl --explain <verb> --json`: outputs project kind, verb, executable, args, cwd, and env.
- Schemas are documented in `docs/json-schema.md`.

## Test Matrix & Status

### Verification Passed
- `cargo fmt --check`: Clean formatting across all files.
- `cargo clippy -- -D warnings`: 0 warnings.
- `cargo test`: 96 tests passing (52 unit tests + 44 integration tests).
  - Config precedence tests
  - Broken config line/col reporting tests
  - Exit code 3 passthrough test
  - Missing tool exit code 127 + install hint test
  - Monorepo depth, discovery, prefixing, serial, and fail-fast tests
  - Ruby rubocop / rspec / rake test detection tests
  - Java Maven & Gradle checkstyle detection tests
  - Docker verb restrictions tests
  - Makefile target merging with Go/Rust/Node tests
  - JSON schema v1 validation tests
- `scripts/smoke.sh` and `scripts/smoke.ps1`: Shallow-clone smoke tests across Vite, Next.js, lodash, ripgrep, glow, FastAPI, Spring Petclinic, and OkHttp.
- `install.sh` and `install.ps1`: Verified cross-platform installation and SHA256 checksum verification (handling `byte[]` in PowerShell 5.1).
