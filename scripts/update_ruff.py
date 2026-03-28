#!/usr/bin/env python3
"""Update ruff git dependency pins in Cargo.toml and minimum version in pyproject.toml."""
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
    pyproject_path = ROOT / "pyproject.toml"

    cargo = cargo_path.read_text()
    current = current_ruff_tag(cargo)
    latest = latest_ruff_tag()

    if latest == current:
        print(f"ruff is already at latest: {current}")
        return

    print(f"Updating ruff: {current} → {latest}")

    version = latest.lstrip("v")
    current_version = current.lstrip("v")

    cargo_path.write_text(cargo.replace(f'tag = "{current}"', f'tag = "{latest}"'))

    pyproject = pyproject_path.read_text()
    pyproject_path.write_text(re.sub(r"ruff>=[\d.]+", f"ruff>={version}", pyproject))

    readme_path = ROOT / "README.md"
    readme = readme_path.read_text()
    readme_path.write_text(re.sub(
        r"!\[ruff [\d.]+\]\(https://img\.shields\.io/badge/ruff-[\d.]+-30173D\)",
        f"![ruff {version}](https://img.shields.io/badge/ruff-{version}-30173D)",
        readme,
    ))

    print("Done. Run `cargo build` to verify the new version compiles.")


if __name__ == "__main__":
    main()
