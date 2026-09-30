# NOTES.md — singularity-cli decisions and status

## Assumptions

- Binary name is `sgl` (set via `[[bin]]` in Cargo.toml).
- `--explain` prints to stderr (same stream as `print_cmd`). Integration tests
  check stderr for explain output and stdout for JSON output.
- `detect_all` checks only the top-level directory. For monorepo `info`, the code
  falls back to `monorepo::discover` when top-level detection returns nothing.
- `Verb::Run(script)` maps directly to the script name for Make/just/npm etc.
  `--explain run` (the global flag) explains the `run` verb with the first unnamed
  positional being the script. In practice, use `sgl --dry-run run <script>` to
  see the resolved command for a specific script.
- `sgl doctor` checks for required tools at the top level only. Monorepo doctor
  would check each sub-project.
- Python `dev` detection uses file-content heuristics for FastAPI/Flask/Django.
  If none match, `dev` returns None and sgl reports no command available.
- Rust `dev` prefers `cargo watch -x run`; falls back to `cargo run` if
  cargo-watch is not installed (detected at runtime via `which`).
- Go `lint` prefers `golangci-lint run`; falls back to `go vet ./...`.
- Color output is gated on: is_terminal && !NO_COLOR && !--no-color.
  Truecolor gradient requires COLORTERM=truecolor or COLORTERM=24bit.

## Decisions

- Used `owo-colors` with `supports-colors` feature for `truecolor()` method.
  Replaced `if_supports_color` pattern with simple boolean gating (cleaner,
  avoids the feature complexity).
- Detectors are ordered: Node > Python > Rust > Go > Java > .NET > Ruby > Docker > Make.
  Language detectors win over tool-file detectors when both exist in the same dir.
- `singularity.toml` config: shell command strings are split on whitespace (no
  shell expansion). This is intentional — no shell injection surface.
- Integration tests use `--explain` and `--dry-run` so no real toolchains are
  required.
- `Verb::from_str` always returns Some (unknown verbs become `Verb::Run(s)`) to
  support arbitrary custom script names in `--explain`.
- Monorepo parallel execution is scaffolded in architecture but the current
  `run_verb` function runs detectors sequentially. Parallel execution via tokio
  tasks would be added in the next iteration (the monorepo flag infra is there).

## Status

### Done
- All source modules: main, cli, banner, ui, resolve, exec, config, doctor,
  monorepo, git
- All 9 detectors: node, python, rust, go, java, dotnet, ruby, docker, make
- Test fixtures: 13 fixture directories
- 41 unit tests (all passing)
- 27 integration tests (all passing)
- cargo fmt, cargo clippy, cargo test all clean
- CI workflow (ci.yml)
- Release workflow (release.yml) with musl Linux + macOS + Windows
- install.sh and install.ps1
- Homebrew formula template (packaging/sgl.rb)
- README.md, CONTRIBUTING.md, LICENSE, CHANGELOG.md, .gitignore
- landing/cli.md
- Shell completions: bash, zsh, fish, powershell

### Stubbed / not yet implemented
- Monorepo parallel execution (architecture present, runs sequentially today)
- `--changed` flag (git.rs helpers written, flag not wired into CLI yet)
- `--serial` and `--fail-fast` flags (not wired yet)
- Doctor in monorepo mode (checks top-level only)
- `sgl doctor --json` output (flag parsed, but doctor.rs uses the json param)

### Next
- Wire `--changed`, `--serial`, `--fail-fast` into the monorepo execution path
- Add tokio::spawn parallel execution for monorepo verbs
- Emit summary table after parallel runs
- Add more detector unit tests (dotnet, ruby verb coverage)
