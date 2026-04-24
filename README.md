# ruffian

[![PyPI](https://img.shields.io/pypi/v/ruffian)](https://pypi.org/project/ruffian/)
[![Wheel](https://img.shields.io/pypi/wheel/ruffian)](https://pypi.org/project/ruffian/)
[![Downloads](https://static.pepy.tech/badge/ruffian)](https://pepy.tech/projects/ruffian)
[![CI](https://github.com/tluolamo/ruffian/actions/workflows/ci.yml/badge.svg)](https://github.com/tluolamo/ruffian/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![ruff 0.15.12](https://img.shields.io/badge/ruff-0.15.12-30173D)

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
ruffian rule PLC0302

# JSON output (same format as ruff --output-format json)
ruffian check src/ --output-format json
```

ruffian accepts the same flags as `ruff check` for the options it passes through. Run `ruffian --help` for the full list.

> **Output format note:** ruffian's text output matches `ruff check --output-format concise --quiet` — one `path:row:col: CODE [*] message` line per violation, no source context or summary footer. Use `--output-format json` for full detail.

---

## Inline suppression

Add a `# ruffian: noqa` comment to suppress ruffian violations on a specific line:

```python
some_huge_module_header = True  # ruffian: noqa           # suppress all ruffian rules on this line
some_huge_module_header = True  # ruffian: noqa PLC0302   # suppress a specific rule
some_huge_module_header = True  # ruffian: noqa PLC0302, RFN001  # suppress multiple rules
```

> **Note:** ruff's own `# noqa` suppression is handled by ruff before violations reach ruffian. Use `# noqa: CODE` to suppress ruff violations and `# ruffian: noqa CODE` to suppress ruffian violations.

---

## Configuration

All ruffian config lives in `pyproject.toml` under `[tool.ruffian]`. Ruff's own `[tool.ruff]` section is untouched and passed directly to ruff.

```toml
[tool.ruffian]
select = ["PLC0302"]   # built-in rules to enable (empty = all enabled)
ignore = []

[tool.ruffian.rules.PLC0302]
max-module-lines = 800   # override the default (1000); `max-lines` also accepted for compatibility

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

### PLC0302 — too-many-module-lines

Reports Python modules that exceed a configurable line count. Encourages splitting large files before they become hard to navigate.

```toml
[tool.ruffian.rules.PLC0302]
max-module-lines = 800   # default: 1000; `max-lines` also accepted for v0.1 compatibility
```

To raise the limit or disable the rule globally:

```toml
[tool.ruffian]
ignore = ["PLC0302"]   # disable entirely
```

To suppress it for a single file, add `# ruffian: noqa PLC0302` to line 1 of that file (the rule always fires on line 1):

```python
# ruffian: noqa PLC0302 — this file is intentionally large (generated code)
...
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

## Editor integration

ruffian's output format is identical to ruff's, so any editor integration that already works with ruff will work with ruffian without changes. Replace `ruff` with `ruffian` in your editor's lint command setting.

For VS Code with the [Ruff extension](https://marketplace.visualstudio.com/items?itemName=charliermarsh.ruff), point it at the ruffian binary:

```json
{
  "ruff.path": ["/path/to/ruffian"]
}
```

---

## GitHub Actions

```yaml
- name: Lint with ruffian
  run: |
    pip install ruffian
    ruffian check src/
```

Or pin the version for reproducible CI:

```yaml
- name: Lint with ruffian
  run: |
    pip install ruffian==0.1.0
    ruffian check src/
```

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

### Proposing a new built-in rule

ruffian only ships rules that ruff has explicitly declined to implement. Before opening a PR here, please follow this process:

1. **Propose the rule to ruff first.** Open a feature request in the [ruff issue tracker](https://github.com/astral-sh/ruff/issues). Many rules belong there, not here — ruff has a much larger audience and faster release cadence.

2. **Use the plugin system in the meantime.** While ruff considers your proposal, implement the rule as a [ruffian plugin](#plugin-system). This lets you and your team use it immediately with zero Rust code and no waiting on anyone.

3. **If ruff declines, open an issue here first.** If ruff closes or explicitly refuses your issue, open a ruffian issue with a link to that discussion. This lets us align on the rule design before any code is written.

4. **Then submit a PR.** Built-in ruffian rules are written in Rust — see [Adding a built-in rule](#adding-a-built-in-rule) below. Your PR must reference the ruffian issue and the upstream ruff discussion that refused the rule.

This keeps ruffian's built-in rule set small and intentional: every rule here has a paper trail explaining why ruff said no.

### Prerequisites

```bash
# 1. Install the task runner (provides the `task` command)
brew install go-task

# 2. Install everything else
task setup
```

`task setup` installs the Rust toolchain via rustup, dev tools (cargo-llvm-cov, llvm-tools), and maturin for packaging. After it completes, restart your terminal or run `source ~/.cargo/env` to pick up the Rust toolchain.

---

### Adding a built-in rule

1. Create `src/rules/<rule_name>.rs` and implement the `Rule` trait:

```rust
pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn check(&self, file: &ParsedFile) -> Vec<Violation>;
}
```

`ParsedFile` exposes three fields:

| Field | Type | Notes |
|-------|------|-------|
| `path` | `String` | Path to the file as provided to ruffian on the CLI |
| `source` | `String` | Raw source text |
| `ast` | `Option<Parsed<ModModule>>` | Parsed AST from `ruff_python_parser`; `None` if the file has syntax errors |

For source-level checks (line count, regex, etc.) use `file.source`. For structural checks use `file.ast.as_ref().map(|p| p.syntax())` to get the `ModModule` and walk the statement list.

2. Register it in `src/rules/mod.rs` — one line in `all_rules()`. No other changes required.

See [PLC0302](src/rules/too_many_module_lines.rs) for a complete example.

### Development commands

```bash
task build              # debug build
task test               # run all tests
task test:coverage      # run tests with coverage report, fail below 85%
task lint               # cargo clippy -D warnings
task lint:fix           # clippy --fix (--allow-dirty) + cargo fmt
task fmt:check          # check formatting without writing changes (what CI runs)
task install:local      # build and install into the active Python env for manual testing
task version:bump       # bump minor version and push (optionally: task version:bump -- 1.0.0)
task release            # interactive release: checks CI, generates notes, creates GitHub release
task ruff:update        # update ruff dependency pins to latest release
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
