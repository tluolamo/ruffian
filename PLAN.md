# ruffian — Project Plan

> *A ruffian breaks rules. This tool adds the ones ruff refused.*

A Rust CLI that acts as a seamless superset of ruff: it runs ruff internally, adds its own built-in rules, and supports user-defined plugin executables — all with a unified output stream indistinguishable from ruff.

---

## Name

**`ruffian`** (`pip install ruffian`, binary: `ruffian`)

- Contains "ruff" — obvious lineage
- A ruffian is a troublemaker who bends the rules — fitting for a tool that ships the rules ruff explicitly refused
- Short, memorable, Googleable

---

## Design Goals

1. **Drop-in superset** — replace `ruff` with `ruffian` in CI scripts, pre-commit hooks, and editor config with minimal changes
2. **Same performance envelope** — ruff and built-in rules run concurrently; total wall time ≈ ruff alone
3. **Extensible rule set** — adding a new built-in rule is a single Rust file; no architectural changes needed
4. **User plugins** — any executable that honors the plugin contract can register as a rule source
5. **Output fidelity** — merged output is sorted by file/line and formatted identically to ruff's text output; JSON mode also available

---

## Architecture Overview

```
ruffian check src/
    │
    ├─── [thread A] spawn `ruff check --output-format json` → parse violations
    │
    ├─── [thread B] parse Python files with ruff_python_parser
    │                └── run each built-in rule → collect violations
    │
    └─── [thread C] discover + invoke plugin executables
                     └── collect JSON violations from each plugin's stdout

    merge + sort all violations → format output → exit code
```

`ruffian format` is a pure passthrough to `ruff format` — no interception needed.
`ruffian check --fix` runs ruff's fix pass first, then re-evaluates custom rules.

---

## Key Technical Decisions

### Parser

Use ruff's own published crates (`ruff_python_ast`, `ruff_python_parser`, `ruff_source_file`) from crates.io. This means:
- Same AST representation as ruff uses internally
- No risk of parser divergence
- Rules can be written using the exact same node types as ruff's source

### Ruff invocation

Call ruff as a subprocess with `--output-format json`. This decouples us from ruff's internal Rust API (which is not stable) while staying compatible with any ruff version the user has installed. The user brings their own ruff — ruffian just orchestrates it.

### Concurrency

Use `tokio` for async subprocess management. Ruff subprocess, built-in rule evaluation, and plugin invocations all run concurrently. File I/O for built-in rules is parallelized with `rayon`.

### Configuration

All ruffian config lives in `pyproject.toml` under `[tool.ruffian]`. Ruff's own config is untouched and read by ruff directly. ruffian reads only its own section.

```toml
[tool.ruffian]
# built-in rules
select = ["PLC0302"]          # too-many-module-lines
ignore = []

[tool.ruffian.rules.PLC0302]
max-lines = 800

# user plugins
[[tool.ruffian.plugins]]
name = "my-custom-rule"
executable = "./scripts/lint_my_rule.py"  # or any executable
config = { threshold = 5 }               # passed to plugin as JSON on stdin
```

---

## Rule Numbering Convention

Built-in ruffian rules use the prefix `RFN` (Ruffian):

| Code    | Description               |
|---------|---------------------------|
| PLC0302  | too-many-module-lines     |
| RFN002  | (next built-in rule)      |
| RFN9xx  | reserved for plugins      |

User plugins assign their own codes (plugin name is the namespace). The plugin contract section below defines how codes are reported.

---

## Plugin Contract

A plugin is **any executable** (Python script, shell script, compiled binary, etc.) that follows this contract:

### Invocation

ruffian calls the plugin once per lint run:

```
./my_plugin.py [file1.py file2.py ...]
```

The list of files to check is passed as positional arguments. ruffian also writes a JSON config blob to the plugin's **stdin**:

```json
{
  "ruffian_version": "0.1.0",
  "config": { "threshold": 5 }
}
```

`config` is whatever the user put in `[[tool.ruffian.plugins]]` → `config`.

### Output

The plugin writes a JSON array to **stdout**. Each violation:

```json
[
  {
    "code": "MY001",
    "message": "Human-readable description of the violation",
    "filename": "/abs/path/to/file.py",
    "location": { "row": 42, "column": 0 },
    "end_location": { "row": 42, "column": 10 },
    "url": "https://my-docs.example.com/rules/MY001",
    "fix": null
  }
]
```

- `code`, `message`, `filename`, `location` are required
- `end_location`, `url`, `fix` are optional (null is fine)
- Empty array `[]` means no violations
- Plugin exits with code `0` regardless of violations found (violations are communicated via the JSON, not the exit code)
- Any output to **stderr** is forwarded to ruffian's stderr with a `[plugin: name]` prefix
- Exit code non-zero → ruffian treats the plugin as failed and reports an error (separate from lint violations)

### Minimal Python plugin example

```python
#!/usr/bin/env python3
import json, sys

config_blob = json.loads(sys.stdin.read())
files = sys.argv[1:]
violations = []

for path in files:
    source = open(path).read()
    lines = source.splitlines()
    # ... your logic here ...

print(json.dumps(violations))
```

---

## Project Structure

```
ruffian/
├── Cargo.toml
├── pyproject.toml          # for maturin/PyO3 packaging
├── src/
│   ├── main.rs             # CLI entry (clap)
│   ├── cli.rs              # subcommand definitions mirroring ruff's interface
│   ├── config.rs           # reads [tool.ruffian] from pyproject.toml
│   ├── runner.rs           # orchestrates ruff subprocess + rules + plugins concurrently
│   ├── output.rs           # merges violations, formats text/JSON output
│   ├── ruff.rs             # ruff subprocess wrapper, JSON output parser
│   ├── plugin.rs           # plugin discovery, invocation, output parsing
│   └── rules/
│       ├── mod.rs          # rule registry — register new rules here, one line each
│       ├── rule.rs         # Rule trait definition
│       └── rfn001_too_many_module_lines.rs
│           ...             # one file per rule
├── tests/
│   ├── fixtures/           # .py files used in integration tests
│   └── integration/
└── python/
    └── example_plugins/    # example plugin scripts shipped with the package
        └── example_plugin.py
```

### The Rule trait

Every built-in rule implements one trait:

```rust
pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn check(&self, file: &ParsedFile) -> Vec<Violation>;
}
```

Adding a new rule = new file in `rules/`, implement the trait, register in `rules/mod.rs`. No other changes.

---

## Distribution

Packaged with **maturin** as a Python wheel containing the compiled Rust binary. Published to PyPI as `ruffian`.

```bash
pip install ruffian
# or
uv add --dev ruffian
```

The wheel includes the `ruffian` binary. Ruff is declared as a dependency in the package metadata so it is installed automatically if not already present.

Users replace `ruff` with `ruffian` everywhere:

```yaml
# .pre-commit-config.yaml  (before)
- repo: https://github.com/astral-sh/ruff-pre-commit
  hooks:
    - id: ruff

# (after)
- repo: local
  hooks:
    - id: ruffian
      name: ruffian
      entry: ruffian check
      language: system
      types: [python]
```

---

## Implementation Phases

### Phase 1 — Scaffold & passthrough (v0.1)

- [x] Cargo project with clap CLI matching ruff's `check` / `format` / `--help` surface
- [x] `ruff format` passthrough (exec ruff directly)
- [x] `ruff check` passthrough via subprocess, parse JSON, re-emit text output
- [x] Output format matches ruff exactly (verified by diff tests) — matches `ruff check --output-format concise --quiet`
- [ ] maturin packaging, basic PyPI publish

Milestone: `ruffian check .` and `ruff check .` produce identical output on a project with no custom rules.

### Phase 2 — First built-in rule (v0.2)

- [x] `ParsedFile` struct wrapping `ruff_python_parser` output — holds raw source + `Option<Parsed<ModModule>>` (populated via `parse_module`; `None` on syntax error)
- [x] `Rule` trait + rule registry
- [x] `PLC0302` too-many-module-lines (configurable, default 1000)
- [x] Config parsing from `pyproject.toml` `[tool.ruffian]`
- [x] Concurrent execution: ruff subprocess + built-in rules run in parallel
- [x] Merged, sorted output
- [x] JSON output mode (`--output-format json`) includes ruffian violations

Milestone: `ruffian check .` runs ruff + PLC0302 and reports both.

### Phase 3 — Plugin system (v0.3)

- [x] Plugin discovery from `[[tool.ruffian.plugins]]`
- [x] Plugin invocation with file args + stdin config JSON
- [x] Plugin stdout parsing, stderr forwarding
- [x] Plugin failure handling (non-zero exit → error, not lint failure)
- [x] Example Python plugin shipped in repo
- [x] Plugin documentation

Milestone: A user can drop a Python script into their repo and register it as a ruffian plugin with zero Rust code.

### Phase 4 — Polish (v0.4+)

- [x] `ruffian rule PLC0302` — print rule docs (mirrors `ruff rule E501`)
- [x] `ruffian check --select PLC0302` / `--ignore PLC0302`
- [x] `# ruffian: noqa PLC0302` inline suppression
- [ ] VS Code problem matcher compatibility (output format already matches ruff's)
- [x] GitHub Actions example in README
- [ ] Second and third built-in rules based on demand

---

## Open Questions

1. **Ruff version pinning** — should ruffian declare a minimum ruff version, or be fully version-agnostic? The JSON output format has been stable, but worth testing against ruff's changelog.
2. **`ruff_python_parser` crate stability** — ruff publishes these crates but does not guarantee API stability. We may need to vendor or pin the version tightly and update deliberately.
3. **Plugin sandboxing** — plugins run as arbitrary executables. Document clearly that users are responsible for trusting their plugins. No sandboxing in v1.
4. **Fix support** — ruff's `--fix` modifies files in place. ruffian rules and plugins would need to emit structured fix suggestions for this to work end-to-end. Defer to a later phase.
