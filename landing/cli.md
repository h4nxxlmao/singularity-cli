# sgl

`sgl` looks at your current directory, detects what kind of project it is, and gives you the same commands regardless of the stack.

## Install

```sh
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/h4nxxlmao/singularity-cli/main/install.sh | sh
```

```powershell
# Windows
irm https://raw.githubusercontent.com/h4nxxlmao/singularity-cli/main/install.ps1 | iex
```

```sh
# Cargo
cargo install --git https://github.com/h4nxxlmao/singularity-cli
```

## What it detects

| Ecosystem | Detection |
|-----------|-----------|
| Node.js | `package.json` |
| Python | `pyproject.toml`, `requirements.txt`, `Pipfile` |
| Rust | `Cargo.toml` |
| Go | `go.mod` |
| Java | `pom.xml`, `build.gradle` |
| .NET | `*.sln`, `*.csproj` |
| Ruby | `Gemfile` |
| Docker | `docker-compose.yml`, `compose.yaml` |
| Make / just / Taskfile | `Makefile`, `justfile`, `Taskfile.yml` |

Within each ecosystem, `sgl` picks the right package manager from the lockfile
(pnpm, yarn, bun, uv, poetry, etc.) without configuration.

## Commands

```
sgl setup       install dependencies
sgl dev         start the dev server or watcher
sgl test        run tests
sgl build       build
sgl lint        lint
sgl fmt         auto-format
sgl run <name>  run a script or task
sgl doctor      check tool versions
sgl info        show what was detected
sgl init        write singularity.toml
```

## FAQ

**Does it need a config file?**
No. It works by detection. Add `singularity.toml` only when you want to override a command.

**Does it replace npm/cargo/make?**
No. It calls them. It is a thin dispatch layer, not a build system.

**What if my project isn't detected?**
Run `sgl info` to see what was found. Add `singularity.toml` to specify commands explicitly, or open an issue to request a detector.

**Does it support monorepos?**
Yes. If it finds more than one project in subdirectories, it runs each verb in parallel with prefixed output. Use `--project <name>` to target one.

**Where does it install to?**
`~/.local/bin` on Unix, `%USERPROFILE%\.local\bin` on Windows. Override with `SGL_INSTALL_DIR`.
