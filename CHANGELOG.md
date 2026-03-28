# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

## [0.1.0] — 2026-03-28

### Added

- `ruffian check` — runs ruff and ruffian built-in rules concurrently; output matches `ruff check --output-format concise --quiet`
- `ruffian format` — pure passthrough to `ruff format`
- `ruffian rule <CODE>` — prints documentation for a built-in rule, mirroring `ruff rule`
- **PLC0302** (`too-many-module-lines`) — reports modules exceeding a configurable line limit (default: 1000, matching pylint C0302)
- `--output-format json` — merged JSON output including ruffian violations alongside ruff's
- `--fix` — passes through to ruff's fix pass
- `--select` / `--ignore` — CLI flags to enable or suppress specific ruffian rules
- Plugin system — any executable honoring the plugin contract can register as a violation source via `[[tool.ruffian.plugins]]` in `pyproject.toml`
- Config via `[tool.ruffian]` in `pyproject.toml` — `select`, `ignore`, per-rule config, and plugin registration
- `ParsedFile` struct — exposes raw source and parsed AST (`ruff_python_parser`) to all built-in rules
- Directory expansion respecting `.gitignore` and ruff's default excludes (`.venv`, `__pycache__`, `dist`, etc.)
