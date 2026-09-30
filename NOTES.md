# NOTES.md — singularity-cli decisions and status

## Architecture & Specifications

- **Binary & Package Name**: The binary is `sgl`. The package name published on crates.io is `singularitycli` (`singularity-cli` was taken by an unrelated project).
- **Streams**: `--explain` prints to stderr (same stream as `print_cmd`). Machine-readable JSON output (`--json`) is printed strictly to stdout.
- **Config Syntax Errors**: `config::load()` captures TOML parse errors with exact file, line, and column coordinates (`singularity.toml:line:col: message`) using `toml::de::Error::span()`.
- **Precedence**: `singularity.toml` overrides detected project commands. When commands are specified under `[commands]`, those exact commands take precedence over any heuristic detection.
- **Signals & Stdin**: Children inherit stdin, stdout, and stderr for interactive commands (e.g. `sgl dev`, REPLs, interactive tests). Signals (Ctrl+C / SIGINT) are intercepted by Tokio signal handlers and propagate termination to the running child processes.
- **Windows Command Resolution**: Resolves relative executable paths (`./script.sh`, `bin/tool`) and checks `PATHEXT` on Windows (`.exe`, `.cmd`, `.bat`) so tools installed via npm/pip (`npm.cmd`, `pytest.exe`) are located and executed directly without cmd invocation issues.
- **Missing Tool Exit Code**: When a resolved binary is missing from PATH, `sgl` exits with code `127` and displays:
  `sgl: '{program}' not found on PATH (needed for: {cmd_display}). Install {tool_name}: {hint}` with platform-specific installation hints (`winget`, `brew`, `apt`).
- **Exit Code Passthrough**: When a child process terminates with a non-zero exit code (e.g., code 3), `sgl` passes that exact exit code through.
- **Structured Unavailable Verb Messages**: Replaced generic "no command found" with structured message:
  ```text
  sgl: no '{verb}' command for this project ({kind}, {pm})
  path: {path}
  reason: {reason}
  available: {available}
  hint: {hint}
  ```
  Applies to execution failures and `sgl --explain <verb>` (both human and JSON modes), exiting with code 1.
- **Command Quoting & Shell Execution**:
  - `singularity.toml` commands are preserved verbatim as raw shell strings without whitespace re-splitting, eliminating quote corruption for nested single/double quotes, spaces in cwd, and arguments with `&` and `%`.
  - Raw shell commands execute through `cmd /C` on Windows (with `./` normalized to `.\` for script execution) and `sh -c` on Unix. Commands built by `sgl` detectors (e.g. `npm run`, `cargo test`) never go through a shell and use argument vectors directly.
  - If a shell command's program resolves to a Windows `.ps1` shim, it is executed via `powershell -NoProfile -ExecutionPolicy Bypass -File <path> <args>`. Missing tools exit 127 with install hints.

## Windows Path & Batch Execution Hardening

- **Batch File Execution**: Windows `.cmd` and `.bat` shims (such as `npm.cmd`, `pnpm.cmd`, `yarn.cmd`, `npx.cmd`) are executed through `cmd /C`.
- **PowerShell .ps1 Shim Execution**: When resolving programs on Windows (such as `npm.ps1` under nvm4w), `resolve_program` searches for `.exe`, `.cmd`, `.bat` next to the resolved path before falling back to the resolved file itself. If only a `.ps1` script exists, it is executed via `powershell -NoProfile -ExecutionPolicy Bypass -File <path> <args>` (falling back to `pwsh` if `powershell` is unavailable). `sgl doctor` also invokes `.ps1` tools through powershell to query `--version`.
- **Space and Quote Escaping**: Because `cmd.exe /C` strips outer quotes if the first character is a quote, each individual argument containing whitespace or quotes is escaped, and the entire remainder after `/C` is double-quoted and passed via `CommandExt::raw_arg`. This ensures paths with spaces (e.g. `D:\vsc save 7\project`) and arguments with spaces work without quote-stripping corruption.
- **Path Normalization**: `normalize_path` canonicalizes forward slashes to backslashes on Windows, resolves relative cwd inputs against `current_dir()`, and strips `\\?\` verbatim prefixes that break third-party tooling.
- **Monorepo & .gitignore**: `WalkBuilder` is configured with `require_git(false)` so `.gitignore` files inside directories (including directories with spaces) are respected even when initialized outside a `.git` root. Subproject walk guards on `entry.depth() == 0 || dir == root` to prevent root self-detection.
- **Doctor Tool Detection**: `sgl doctor` uses `which` with `resolve_program` to locate tools, and invokes `.cmd` / `.bat` / `.ps1` tools through their appropriate shell wrappers to extract version information (e.g. `npm --version`).

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
- **Fail-Fast**: `--fail-fast` halts execution immediately upon the first project failure, cancelling pending tasks and terminating child processes.
- **Changed-Only**: `--changed` filters sub-projects based on Git modifications (staged, unstaged, and untracked files compared against HEAD). Paths from `git diff` / `git status` are normalized against Windows forward-to-backslash discrepancies and 8.3 short-path conversions.
- **Prefix Streaming**: For non-dev verbs running in parallel, project output lines are buffered and streamed prefixed with `[<project>]`.
- **Dev Mode**: `sgl dev` runs all workspace dev servers concurrently until interrupted by Ctrl+C.
- **Summary Table**: Emits a formatted summary table displaying `Project`, `Verb`, `Status` (success, failed, or skipped), and `Duration`. Exits non-zero if any project fails.
- **Flag Verification**: `--serial`, `--changed`, and `--fail-fast` are verified through live execution integration tests against multi-project Git repos. All flags are documented in `sgl --help`.
- **Project Filtering (`--project`)**:
  - Matches projects by either project name (`p.name`) or relative path (`./packages/web`, `packages/web`).
  - Works regardless of flag position (`sgl test --project foo` and `sgl --project foo test`).
  - When an unknown project is supplied, lists all available project names: `sgl: no project named '{target}' found. Valid projects: {names}` and exits with code 1.

## Machine-Readable JSON Output

All JSON outputs include `"schema_version": 1`:
- `sgl info --json`: outputs detected project kind, runner, resolved verbs, root directory, and monorepo packages.
- `sgl doctor --json`: outputs structured diagnostic checks with tool names, installed versions, required versions, status, and fix suggestions. Never prints 0 bytes even when run on an empty directory (always valid JSON with `tools: []`).
- `sgl --explain <verb> --json`: outputs project kind, verb, executable, args, cwd, and env (or structured error on unavailable verb).
- Schemas are documented in `docs/json-schema.md`.

## Doctor Monorepo Integration

- `sgl doctor` in a monorepo discovers sub-projects using the exact same hierarchy discovery as `sgl info`.
- Tool requirements across all sub-projects are deduplicated so shared dependencies are checked exactly once.
- `sgl doctor --project <NAME/PATH>` filters requirements strictly to the targeted project.

## Test Matrix & Status

### Verification Passed
- `cargo fmt --check`: Clean formatting across all files.
- `cargo clippy -- -D warnings`: 0 warnings.
- `cargo test`: 102 tests passing (53 unit tests + 49 integration tests).
  - Windows `.cmd` shim execution and raw quote preservation
  - CWD containing spaces ("vsc save 7/hanx.pro-v2") across info, doctor, explain
  - Spaced monorepo walking with `.gitignore` exclusion (`require_git(false)`)
  - Missing tool exits 127 naming program and per-OS hints
  - Node missing script structured error (both execution and `--explain`)
  - Docker lint unavailable structured error
  - Maven lint unavailable structured error
  - Broken config line/col reporting tests
  - Exit code 3 passthrough test
  - Monorepo depth, discovery, prefixing, serial, and fail-fast tests
  - Ruby rubocop / rspec / rake test detection tests
  - Java Maven & Gradle checkstyle detection tests
  - Docker verb restrictions tests
  - Makefile target merging with Go/Rust/Node tests
  - JSON schema v1 validation tests
- `scripts/smoke.sh` and `scripts/smoke.ps1`: Shallow-clone smoke tests across Vite, Next.js, lodash, ripgrep, glow, FastAPI, Spring Petclinic, and OkHttp.
- `install.sh` and `install.ps1`: Verified cross-platform installation and SHA256 checksum verification (handling `byte[]` in PowerShell 5.1).

### Hardening Pass: Item 6 (`--quiet` Payload Output Preservation)
- `--quiet` suppresses banner and progress chatter (`ui.print_cmd` during execution, verb listing, etc.).
- Preserved payload output:
  - `info`: Removed `if !ui.quiet` in `print_info`, ensuring detected projects tree/list and empty project messages always print.
  - `doctor`: Table rows are printed with colored output without gating through `ui.success` (which suppressed "ok" rows in quiet mode).
  - `--explain`: Plan details (`explain_plan`) and unavailable error messages are always printed.
  - `--dry-run`: Planned commands are printed even when `--quiet` is enabled, both in single project and monorepo modes.
  - Monorepo summary table: Result rows are printed directly with coloring so they are not silenced in quiet mode.
  - Errors: `ui.error` and stderr error messages remain visible unconditionally.
- Integration tests in `tests/integration.rs::quiet_flag_hides_banner_but_prints_payloads` verify all 5 aspects.

### Hardening Pass: Item 7 (`schema_version: 1` & JSON Documentation)
- Verified and tested `schema_version: 1` present in top-level JSON responses for:
  - `sgl info --json`
  - `sgl doctor --json` (standard, empty directories, and missing project filter error cases)
  - `sgl --explain <verb> --json` (successful plan generation and unavailable verb error cases)
- Updated `docs/json-schema.md` to document the optional `error` field in `SglDoctor` and `SglExplain` when filter or verb resolution fails.
- Integration test in `tests/integration.rs::schema_version_present_in_all_json_commands` tests all commands and failure modes.

### Hardening Pass: Item 8 (`run` and Extra Args Passthrough with npm `--` separator)
- Implemented `CommandPlan::append_extra_args(&mut self, extra_args: &[String])` to handle forwarding arguments to underlying commands:
  - Detects if `program` is `npm` (or `npm.cmd`) executing a script or test (`["run", ...]`, `["test"]`), or if `raw_shell` runs `npm run ...` / `npm test`.
  - Automatically inserts the required `--` separator before forwarded arguments unless already present.
  - Correctly updates both `args` and `raw_shell` (for commands defined in `singularity.toml`).
- Used across both single project execution (`main.rs`) and monorepo execution (`monorepo.rs`).
- Integration tests in `tests/integration.rs::npm_run_and_extra_args_passthrough` verify:
  - `sgl run test -- --watch` generates `npm run test -- --watch`
  - `sgl test -- --watch` generates `npm run test -- --watch`

### Hardening Pass: Item 9 (Single Shared "Not Available" Message)
- Standardized all unavailable verb messages through `resolve::format_unavailable_message`:
  - 0 matches for "no command found" or "no commands found" in the codebase.
  - Consistent 5-line structured message across single projects, monorepo roots, dotnet projects, and make/justfile projects:
    - Line 1: `sgl: no '<verb>' command for this project (<kind>[, <pm>])`
    - Line 2: `path: <dir>`
    - Line 3: `reason: <detailed reason>`
    - Line 4: `available: <available commands/scripts/recipes>`
    - Line 5: `hint: <actionable hint>`
  - Fixed DotNet supported verbs (removed unsupported `dev`).
  - Integration tests in `tests/integration.rs::unavailable_message_format_consistent_everywhere` verify exact structure for `dotnet dev` and directories with no project detected.

### Hardening Pass: Item 10 (`justfile` Recipe Parsing)
- Implemented `parse_just_recipes` in `src/detect/make.rs`:
  - Parses recipe names directly from `justfile` and `Justfile` without requiring `just` binary installed.
  - Correctly ignores comments, variable assignments (`:=`), settings, exports, aliases, indented recipe body lines, private recipes (`[private]` and `_` prefixes).
  - Handles parameter signatures (e.g. `test filter=""`, `deploy target:`).
  - Maps standard verbs (`test`, `build`, `fmt`, `lint`, `dev`, `setup`) to matching recipes.
  - Exposes other recipes via `sgl run <recipe>` and includes them in `available:` lists on unavailable messages.
  - Integrates with other detectors when a `justfile` is present in a Go/Rust/Node directory.
- Unit tests in `src/detect/make.rs::detects_justfile_and_recipes` and integration test `tests/integration.rs::justfile_recipe_parsing_and_execution` verify full functionality.




