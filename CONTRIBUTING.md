# Contributing

## Setup

```sh
git clone https://github.com/singularity-cli/singularity-cli
cd singularity-cli
cargo build
```

## Checks

Run before submitting a PR:

```sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Adding a detector

See the "Adding a detector" section in README.md.

Each detector lives in `src/detect/<lang>.rs` and implements the `Detector` trait.
Add it to `all_detectors()` in `src/detect/mod.rs`.
Add fixtures under `tests/fixtures/<lang>/` and integration tests in `tests/integration.rs`.

## Test fixtures

Fixtures are small directories that mimic a real project. They contain only the files
needed for detection (e.g. `package.json`, `Cargo.toml`). No real dependencies.
Tests use `--explain` and `--dry-run` so no toolchains are required to run the tests.

## Commits

Use short, imperative messages: `add go detector`, `fix pnpm lockfile detection`.

## Pull requests

- Tests must pass (`cargo test`).
- No clippy warnings (`cargo clippy -- -D warnings`).
- Formatted (`cargo fmt`).
