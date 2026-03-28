# ruffian

> *A ruffian breaks rules. This tool adds the ones ruff refused.*

**ruffian** is a drop-in superset of [ruff](https://github.com/astral-sh/ruff). It runs ruff internally, adds its own built-in lint rules, and supports user-defined plugin executables — all producing output that is indistinguishable from ruff's own.

Replace `ruff` with `ruffian` in your CI scripts, pre-commit hooks, and editor config. Everything ruff does still works. ruffian adds on top.

---

## Installation

```bash
pip install ruffian
# or
uv add --dev ruffian
```

ruff is a declared dependency and will be installed automatically.

---

## Usage

```bash
# Check files (ruff rules + ruffian built-in rules + your plugins)
ruffian check src/

# Format files — pure passthrough to ruff format
ruffian format src/

# Show documentation for a built-in rule
ruffian rule RFN001

# JSON output (same format as ruff --output-format json)
ruffian check src/ --output-format json
```

ruffian accepts the same flags as `ruff check` for the options it passes through. Run `ruffian --help` for the full list.

---

## Configuration

All ruffian config lives in `pyproject.toml` under `[tool.ruffian]`. Ruff's own `[tool.ruff]` section is untouched and passed directly to ruff.

```toml
[tool.ruffian]
select = ["RFN001"]   # built-in rules to enable (empty = all enabled)
ignore = []

[tool.ruffian.rules.RFN001]
max-lines = 800       # override the default (1000)

# User plugins
[[tool.ruffian.plugins]]
name = "no-todo"
executable = "./scripts/no_todo.py"   # any executable
config = {}                            # passed to the plugin as JSON on stdin
```

---

## Built-in rules

| Code    | Name                  | Pylint source | Default |
|---------|-----------------------|---------------|---------|
| PLC0302 | too-many-module-lines | C0302         | 1000    |

### Rule code prefixes

ruffian follows ruff's own prefix conventions. Since ruffian only implements rules that ruff has not, there is no overlap.

| Prefix | Meaning |
|--------|---------|
| `PLC`, `PLE`, `PLR`, `PLW` | Pylint rules not implemented by ruff — same `PL` prefix ruff uses, no conflict since we only ship what ruff skipped |
| `RFN` | Novel rules with no equivalent in any existing linter |
| `RFC` | Reserved for user plugin rules — plugin authors must use this prefix to avoid collisions with ruffian built-ins |

### RFN001 — too-many-module-lines

Reports Python modules that exceed a configurable line count. Encourages splitting large files before they become hard to navigate.

```toml
[tool.ruffian.rules.RFN001]
max-lines = 800   # default: 1000
```

---

## Plugin system

Any executable — Python script, shell script, compiled binary — can act as a ruffian plugin. This is how you add project-specific rules without writing any Rust.

### How ruffian calls your plugin

```
./my_plugin.py file1.py file2.py ...
```

Files to check are passed as positional arguments. A JSON config blob is written to **stdin**:

```json
{
  "ruffian_version": "0.1.0",
  "config": { "threshold": 5 }
}
```

`config` is whatever you put in `[[tool.ruffian.plugins]]` → `config`.

### What your plugin must write to stdout

A JSON array of violations. Empty array means no violations.

```json
[
  {
    "code": "MY001",
    "message": "Human-readable description",
    "filename": "/abs/path/to/file.py",
    "location": { "row": 42, "column": 0 },
    "end_location": { "row": 42, "column": 10 },
    "url": "https://my-docs.example.com/rules/MY001",
    "fix": null
  }
]
```

`code`, `message`, `filename`, and `location` are required. `end_location`, `url`, and `fix` are optional.

**Exit codes:** exit `0` regardless of violations found — violations are communicated via JSON, not the exit code. Exit non-zero to signal that the plugin itself failed (ruffian will report an error, separate from any lint violations).

**Stderr:** any output to stderr is forwarded to ruffian's stderr with a `[plugin: name]` prefix.

### Minimal Python plugin

```python
#!/usr/bin/env python3
import json, sys

config_blob = json.loads(sys.stdin.read())
files = sys.argv[1:]
violations = []

for path in files:
    source = open(path).read()
    # ... your logic ...

print(json.dumps(violations))
```

A working example is in [`python/example_plugins/example_plugin.py`](python/example_plugins/example_plugin.py).

### Security note

Plugins run as arbitrary executables with the same permissions as your shell. Only register plugins you trust.

---

## Pre-commit

```yaml
# .pre-commit-config.yaml
- repo: local
  hooks:
    - id: ruffian
      name: ruffian
      entry: ruffian check
      language: system
      types: [python]
```

---

## Contributing

### Adding a built-in rule

1. Create `src/rules/rfnXXX_<rule_name>.rs` and implement the `Rule` trait:

```rust
pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn check(&self, file: &ParsedFile) -> Vec<Violation>;
}
```

2. Register it in `src/rules/mod.rs` — one line in `all_rules()`. No other changes required.

See [RFN001](src/rules/rfn001_too_many_module_lines.rs) for a complete example.

### Development commands

```bash
task build        # debug build
task test         # run all tests
task lint         # cargo clippy -D warnings
task lint:fix     # clippy --fix + cargo fmt
```

### Coding style

- Functional-style Rust where it doesn't fight the type system
- No traits or structs for things with only one implementation
- Files stay under 500 lines — split by responsibility when they grow
- `thiserror` for error types; `anyhow` only at the binary boundary
- All PRs must pass `cargo clippy -- -D warnings` and `cargo fmt -- --check`

---

## License

[MIT](LICENSE)
