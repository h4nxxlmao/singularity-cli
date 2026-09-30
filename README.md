# singularity-cli

```
     _                   _            _ _
 ___(_)_ __   __ _ _   _| | __ _ _ __(_) |_ _   _
/ __| | '_ \ / _` | | | | |/ _` | '__| | __| | | |
\__ \ | | | | (_| | |_| | | (_| | |  | | |_| |_| |
|___/_|_| |_|\__, |\__,_|_|\__,_|_|  |_|\__|\__, |
             |___/                          |___/
```

`sgl` detects your project type and maps a common set of verbs to the right underlying tool.

## Install

> **Note:** The official site [getsingularity.lol](https://getsingularity.lol/cli) is coming soon. Use the direct GitHub scripts or Cargo install below:

### Shell (Linux / macOS)

```sh
curl -fsSL https://raw.githubusercontent.com/h4nxxlmao/singularity-cli/main/install.sh | sh
```

### PowerShell (Windows)

```powershell
irm https://raw.githubusercontent.com/h4nxxlmao/singularity-cli/main/install.ps1 | iex
```

### Cargo

Install directly with Cargo:

```sh
# From crates.io
cargo install singularitycli

# From Git repository
cargo install --git https://github.com/h4nxxlmao/singularity-cli

# From local checkout
cargo install --path .
```

*(Once the site is live, `curl -fsSL https://getsingularity.lol/cli/install.sh | sh` will also be available.)*

## Commands

| Command | Description |
|---------|-------------|
| `sgl setup` | Install dependencies |
| `sgl dev` | Start dev server or watcher |
| `sgl test [args]` | Run tests |
| `sgl build` | Build |
| `sgl lint` | Lint and format check |
| `sgl fmt` | Auto-format |
| `sgl run <script> [args]` | Run a project script or task (forwards extra args; inserts npm `--` separator) |
| `sgl doctor` | Check tool versions |
| `sgl info [--json]` | Show detected projects |
| `sgl init` | Write `singularity.toml` |
| `sgl completions <shell>` | Print shell completions |

Global flags: `--dry-run`, `--verbose`, `--quiet`, `--no-color`, `--cwd <path>`, `--project <name>`, `--changed`, `--serial`, `--fail-fast`, `--explain <verb>`, `--json`.

## Supported ecosystems

| Ecosystem | Detected by | Package managers |
|-----------|-------------|-----------------|
| Node.js | `package.json` | npm, pnpm, yarn, bun (from lockfile) |
| Python | `pyproject.toml`, `requirements.txt`, `Pipfile` | uv, poetry, pipenv, pip |
| Rust | `Cargo.toml` | cargo |
| Go | `go.mod` | go |
| Java | `pom.xml`, `build.gradle[.kts]` | mvn, gradle/gradlew |
| .NET | `*.sln`, `*.csproj` | dotnet |
| Ruby | `Gemfile` | bundler |
| Docker | `docker-compose.yml`, `compose.yaml` | docker |
| Make / Just | `Makefile`, `justfile`, `Justfile`, `Taskfile.yml` | make, just (recipes parsed zero-config), task |

## Monorepo usage

When `sgl` finds multiple projects in subdirectories (max depth 4, respecting `.gitignore`):

- `sgl info` shows them as a list.
- Any verb runs in each project in parallel (bounded by CPU count), with output prefixed `[project-name]`.
- A summary table shows status and duration per project.

Flags: `--project <name>` to target one, `--serial`, `--fail-fast`, `--changed` (only projects with uncommitted changes vs. base branch).

## singularity.toml reference

Optional. Create with `sgl init` or write by hand.

```toml
[project]
name = "api"

[commands]
test = "pytest -x"
dev = ["docker compose up -d db", "uvicorn app:app --reload"]
deploy = "./scripts/deploy.sh"

[env]
DATABASE_URL = "postgres://localhost/dev"
```

- `[commands]` overrides any verb or adds new scripts accessible via `sgl run <name>`.
- Values can be a string or a list (runs in sequence). Commands are run through a shell (`cmd /C` on Windows, `sh -c` on Unix) preserving quotes, pipes, and environment syntax without re-splitting.
- `[env]` applies to all commands.
- Config errors include the file path and line number.

## Comparison

| | sgl | mise | just | Taskfile |
|--|--|--|--|--|
| Zero-config detection | yes | no | no | no |
| Wraps existing tools | yes | yes | no | no |
| Per-project overrides | yes | yes | yes | yes |
| Shell syntax in tasks | no | no | yes | yes |
| Monorepo support | yes | partial | no | no |

`sgl` is not a replacement for `just` or `Taskfile` for complex task graphs. It is a detection-first dispatch layer.

## Adding a detector

1. Create `src/detect/<lang>.rs`.
2. Define a struct and implement the `Detector` trait:
   - `id()` — unique string name.
   - `detect(dir)` — return `Some(Project)` if files match.
   - `plan(project, verb)` — return `Some(CommandPlan)` for each verb.
   - `required_tools(project)` — list of tools for `sgl doctor`.
3. Add the struct to `all_detectors()` in `src/detect/mod.rs`.
4. Add fixtures to `tests/fixtures/<lang>/` and tests to `tests/integration.rs`.
5. Run `cargo test` and fix failures.

## License

MIT

