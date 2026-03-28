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
- Require 85% line coverage across the project (`task test:coverage`); threshold is project-wide, not per file
- Coverage tool: `cargo-llvm-cov` — install once with `cargo install cargo-llvm-cov`

## Taskfile

- All common commands (build, test, lint, publish) have corresponding Taskfile tasks
- Use `task` to run anything; do not rely on bare `cargo` invocations in documentation

## Rule naming conventions

Ruffian follows ruff's own conventions as closely as possible.

### Prefixes

| Prefix | Used for | Example |
|--------|----------|---------|
| `PLC`, `PLE`, `PLR`, `PLW` | Rules sourced from pylint that ruff has not implemented — follow ruff's `PL` convention exactly | `PLC0302` |
| `RFN` | Novel built-in rules with no upstream source in any existing linter | `RFN001` |
| `RFC` | User-defined plugin rules — **reserved for plugin authors, never used by ruffian itself** | `RFC001` |

- If a rule exists in pylint and ruff hasn't implemented it, use the `PL`-prefixed pylint code (`PLC0302`, `PLR0914`, etc.). Since ruffian only implements rules ruff skipped, there is no overlap with ruff's own `PL` codes.
- Only use `RFN` for rules that have no equivalent in any existing linter.
- Plugin authors **must** use the `RFC` prefix to avoid collisions with ruffian built-ins.

### File names

Snake_case rule name, no code prefix — mirrors ruff's own convention:
- `too_many_module_lines.rs` ✓ (ruff uses `too_many_arguments.rs`, not `plr0913_too_many_arguments.rs`)

### Adding a rule checklist

1. Determine the correct prefix (pylint code → use it; novel → `RFN`)
2. Name the file after the rule name in snake_case
3. Add a comment at the top of the file noting the pylint/upstream source if one exists
4. Register one line in `src/rules/mod.rs`

## Contributing

- One rule per file in `src/rules/`; register it in `src/rules/mod.rs` — no other changes required
- The plugin contract (stdin/stdout JSON format, exit code semantics) must not be broken without a major version bump
- **Version is the single source of truth in `Cargo.toml`** — `pyproject.toml` uses `dynamic = ["version"]` and maturin reads the version from Cargo.toml automatically. Only bump the version in `Cargo.toml`.
