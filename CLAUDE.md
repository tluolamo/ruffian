# ruffian

> A Rust CLI that acts as a seamless superset of ruff: it runs ruff internally, adds its own built-in rules, and supports user-defined plugin executables — all with a unified output stream indistinguishable from ruff.

## Project structure

- `src/` — Rust source code
  - `main.rs` — CLI entry point (clap)
  - `cli.rs` — subcommand definitions mirroring ruff's interface
  - `config.rs` — reads `[tool.ruffian]` from `pyproject.toml`
  - `runner.rs` — orchestrates ruff subprocess + rules + plugins concurrently
  - `output.rs` — merges violations, formats text/JSON output
  - `ruff.rs` — ruff subprocess wrapper, JSON output parser
  - `plugin.rs` — plugin discovery, invocation, output parsing
  - `rules/` — one file per rule; `mod.rs` is the registry
- `tests/` — integration tests with `.py` fixtures
- `python/example_plugins/` — example plugin scripts

## Coding rules

- Write clean, simple, functional-style code; prefer free functions over methods where it does not fight the type system
- Avoid unnecessary abstractions — do not create traits, structs, or enums for things that only have one implementation
- Keep files under 500 lines; split by responsibility when they grow
- Imports at the top of each file
- Do not add `pub` to things that don't need to be public
- Avoid `unwrap()` and `expect()` in library code; use `?` and proper error propagation
- Prefer `thiserror` for defining error types; use `anyhow` only at the top-level binary boundary
- Do not add comments unless the logic is genuinely non-obvious

## Linting

- After making changes, run `cargo clippy -- -D warnings` and fix all warnings
- Format with `cargo fmt` before committing

## Tests

- Integration tests live in `tests/integration/` and use `.py` fixture files from `tests/fixtures/`
- Aim for coverage of all public-facing behaviour via integration tests; unit-test only logic that is hard to reach via the integration path
- Start with happy path, then edge cases
- Each test should assert one outcome; avoid multiple unrelated assertions per test
- Do not test dead code — if code is unreachable from any test entry point, consider removing it

## Taskfile

- All common commands (build, test, lint, publish) have corresponding Taskfile tasks
- Use `task` to run anything; do not rely on bare `cargo` invocations in documentation

## Contributing

- One rule per file in `src/rules/`; register it in `src/rules/mod.rs` — no other changes required
- Plugin contract is documented in `PLAN.md` and must not be broken without a major version bump
- All public API changes should be reflected in `CHANGELOG.md`
