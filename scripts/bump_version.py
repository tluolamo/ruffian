#!/usr/bin/env python3
"""Bump the version in Cargo.toml (single source of truth).

Usage:
  bump_version.py          # auto-increment minor (0.1.0 → 0.2.0)
  bump_version.py 0.3.0    # set explicit version
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent


def read_current(cargo: str) -> str:
    m = re.search(r'^version = "([^"]+)"', cargo, re.MULTILINE)
    if not m:
        print("error: could not find version in Cargo.toml", file=sys.stderr)
        sys.exit(1)
    return m.group(1)


def bump_minor(version: str) -> str:
    major, minor, patch = version.split(".")
    return f"{major}.{int(minor) + 1}.0"


def main() -> None:
    cargo_path = ROOT / "Cargo.toml"
    cargo = cargo_path.read_text()
    current = read_current(cargo)

    if len(sys.argv) == 2 and sys.argv[1] == "--current":
        print(current)
        return

    if len(sys.argv) > 2:
        print("usage: bump_version.py [new-version]", file=sys.stderr)
        sys.exit(1)

    if len(sys.argv) == 2:
        new_version = sys.argv[1].lstrip("v")
        if not re.fullmatch(r"\d+\.\d+\.\d+", new_version):
            print(f"error: version must be X.Y.Z, got: {new_version}", file=sys.stderr)
            sys.exit(1)
    else:
        new_version = bump_minor(current)

    cargo_path.write_text(
        cargo.replace(f'version = "{current}"', f'version = "{new_version}"', 1)
    )
    print(f"Bumped: {current} → {new_version}", file=sys.stderr)
    print(new_version)


if __name__ == "__main__":
    main()
