#!/usr/bin/env python3
"""Update the embedded ruff parser dependencies in Cargo.toml."""
import json
import re
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).parent.parent


def latest_ruff_tag() -> str:
    url = "https://api.github.com/repos/astral-sh/ruff/releases/latest"
    with urllib.request.urlopen(url) as r:
        return json.loads(r.read())["tag_name"]


def current_ruff_tag(cargo: str) -> str:
    m = re.search(r'git = "https://github\.com/astral-sh/ruff",\s*tag = "([^"]+)"', cargo)
    if not m:
        print("error: could not find ruff tag in Cargo.toml", file=sys.stderr)
        sys.exit(1)
    return m.group(1)


def main() -> None:
    cargo_path = ROOT / "Cargo.toml"
    cargo = cargo_path.read_text()
    current = current_ruff_tag(cargo)
    latest = latest_ruff_tag()

    if latest == current:
        print(f"ruff is already at latest: {current}")
        return

    print(f"Updating ruff: {current} → {latest}")

    cargo_path.write_text(re.sub(
        r'(git = "https://github\.com/astral-sh/ruff",\s*tag = ")'
        + re.escape(current) + r'("[\s}])',
        lambda m: f"{m.group(1)}{latest}{m.group(2)}",
        cargo,
    ))
    print("Done. Run `cargo build` to verify the new version compiles.")


if __name__ == "__main__":
    main()
