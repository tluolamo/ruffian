#!/usr/bin/env python3
"""
Example ruffian plugin — reports any Python file that contains the word "TODO".

Usage in pyproject.toml:
    [[tool.ruffian.plugins]]
    name = "no-todo"
    executable = "./python/example_plugins/example_plugin.py"
    config = {}
"""
import json
import sys


def check_file(path: str) -> list[dict]:
    try:
        source = open(path).read()
    except OSError:
        return []

    violations = []
    for row, line in enumerate(source.splitlines(), start=1):
        col = line.find("TODO")
        if col != -1:
            violations.append({
                "code": "PLG001",
                "message": "TODO comment found",
                "filename": path,
                "location": {"row": row, "column": col},
                "end_location": {"row": row, "column": col + 4},
                "url": None,
                "fix": None,
            })
    return violations


def main() -> None:
    config_blob = json.loads(sys.stdin.read())  # noqa: F841 — available for custom config use
    files = sys.argv[1:]
    violations = []
    for path in files:
        violations.extend(check_file(path))
    print(json.dumps(violations))


if __name__ == "__main__":
    main()
