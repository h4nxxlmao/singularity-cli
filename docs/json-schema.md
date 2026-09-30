# singularity-cli (`sgl`) JSON Schema Reference

All JSON outputs from `sgl` are stable, machine-readable interfaces intended for editor extensions, MCP servers, and CI pipelines.

Every JSON payload includes a top-level `schema_version` integer (starting at `1`). Breaking changes to any payload structure will increment `schema_version`.

---

## 1. `sgl info --json`

Provides project detection details for the current directory or monorepo workspace.

### Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SglInfo",
  "type": "object",
  "required": ["schema_version", "directory", "has_config", "projects"],
  "properties": {
    "schema_version": {
      "type": "integer",
      "description": "Schema version (currently 1)"
    },
    "directory": {
      "type": "string",
      "description": "Absolute path to the inspected directory"
    },
    "has_config": {
      "type": "boolean",
      "description": "Whether a singularity.toml file was loaded"
    },
    "projects": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "root", "kind", "package_manager", "scripts"],
        "properties": {
          "name": {
            "type": "string",
            "description": "Detected project name"
          },
          "root": {
            "type": "string",
            "description": "Absolute path to the project root"
          },
          "kind": {
            "type": "string",
            "enum": [
              "node",
              "python",
              "rust",
              "go",
              "make",
              "docker",
              "java",
              "dotnet",
              "ruby"
            ],
            "description": "Detected ecosystem kind"
          },
          "package_manager": {
            "type": ["string", "null"],
            "description": "Detected package manager (e.g. npm, pnpm, uv, cargo, bundler)"
          },
          "scripts": {
            "type": "object",
            "additionalProperties": { "type": "string" },
            "description": "Available scripts or tasks extracted from the project manifest"
          }
        }
      }
    }
  }
}
```

### Example

```json
{
  "schema_version": 1,
  "directory": "/workspace/my-app",
  "has_config": false,
  "projects": [
    {
      "name": "my-app",
      "root": "/workspace/my-app",
      "kind": "node",
      "package_manager": "pnpm",
      "scripts": {
        "build": "vite build",
        "dev": "vite",
        "lint": "eslint ."
      }
    }
  ]
}
```

---

## 2. `sgl doctor --json`

Reports required tools, installed versions, and resolution statuses.

### Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SglDoctor",
  "type": "object",
  "required": ["schema_version", "tools"],
  "properties": {
    "schema_version": {
      "type": "integer",
      "description": "Schema version (currently 1)"
    },
    "tools": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["tool", "found_version", "required_version", "status", "fix"],
        "properties": {
          "tool": {
            "type": "string",
            "description": "Name of the required executable"
          },
          "found_version": {
            "type": ["string", "null"],
            "description": "Installed version detected on PATH, or null if missing"
          },
          "required_version": {
            "type": "string",
            "description": "Semver requirement specified by the project (empty if any version is acceptable)"
          },
          "status": {
            "type": "string",
            "enum": ["ok", "missing", "version mismatch"]
          },
          "fix": {
            "type": ["string", "null"],
            "description": "Installation or upgrade hint if status != ok"
          }
        }
      }
    }
  }
}
```

### Example

```json
{
  "schema_version": 1,
  "tools": [
    {
      "tool": "node",
      "found_version": "20.10.0",
      "required_version": ">=18.0.0",
      "status": "ok",
      "fix": null
    },
    {
      "tool": "pnpm",
      "found_version": "9.1.0",
      "required_version": "",
      "status": "ok",
      "fix": null
    }
  ]
}
```

---

## 3. `sgl --explain <verb> --json`

Resolves the command plan for `<verb>` and returns exact command details without execution.

### Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "SglExplain",
  "type": "object",
  "required": ["schema_version", "verb", "plans"],
  "properties": {
    "schema_version": {
      "type": "integer",
      "description": "Schema version (currently 1)"
    },
    "verb": {
      "type": "string",
      "description": "The verb requested"
    },
    "plans": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["program", "args", "cwd", "env", "reason"],
        "properties": {
          "program": {
            "type": "string",
            "description": "Executable binary to invoke"
          },
          "args": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Command line arguments"
          },
          "cwd": {
            "type": "string",
            "description": "Working directory for the command"
          },
          "env": {
            "type": "object",
            "additionalProperties": { "type": "string" },
            "description": "Environment variable overrides"
          },
          "reason": {
            "type": "string",
            "description": "Explanation of why this command was selected"
          }
        }
      }
    }
  }
}
```

### Example

```json
{
  "schema_version": 1,
  "verb": "test",
  "plans": [
    {
      "program": "cargo",
      "args": ["test"],
      "cwd": "/workspace/my-rust-crate",
      "env": {},
      "reason": "rust: cargo test: Cargo.toml found"
    }
  ]
}
```
