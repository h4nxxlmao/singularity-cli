# Changelog

## Unreleased

### Added
- Initial release.
- Detection for Node.js (npm, pnpm, yarn, bun), Python (uv, poetry, pipenv, pip),
  Rust, Go, Java (Maven, Gradle), .NET, Ruby, Docker Compose, Make/just/Taskfile.
- Verbs: setup, dev, test, build, lint, fmt, run, doctor, info, init, completions.
- `singularity.toml` config with per-verb overrides, command lists, and env vars.
- Monorepo discovery with parallel execution and `--project`, `--serial`, `--fail-fast`, `--changed` flags.
- `--explain <verb>`: print resolved commands without running them.
- `--dry-run`: print all commands and exit zero.
- `sgl doctor`: table of required tools with install hints.
- `sgl info --json`: machine-readable project info.
- Shell completions for bash, zsh, fish, and PowerShell.
- CI workflow (ubuntu, macos, windows).
- Release workflow (Linux musl x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64).
- `install.sh` and `install.ps1`.
- Homebrew formula template.
