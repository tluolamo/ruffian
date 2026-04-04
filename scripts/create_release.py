#!/usr/bin/env python3
"""Create a GitHub release for ruffian.

Flow:
  1. Read version from Cargo.toml
  2. Check if the tag already exists on GitHub
     - If it does, suggest a minor bump and ask for confirmation
     - If confirmed, bump, commit, push
  3. Wait for CI to pass on the pushed commit
  4. Auto-generate release notes via GitHub API
  5. Open $EDITOR so you can review / rewrite them
  6. Create the GitHub release (which triggers the publish workflow)
"""
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).parent.parent


# ── helpers ──────────────────────────────────────────────────────────────────

def cargo_version() -> str:
    m = re.search(r'^version = "([^"]+)"', (ROOT / "Cargo.toml").read_text(), re.MULTILINE)
    if not m:
        abort("could not find version in Cargo.toml")
    return m.group(1)


def bump_minor(version: str) -> str:
    major, minor, _ = version.split(".")
    return f"{major}.{int(minor) + 1}.0"


def gh(*args: str, input: str | None = None) -> dict | list | str:
    """Run a gh command and return parsed JSON output."""
    result = subprocess.run(
        ["gh", *args],
        capture_output=True,
        text=True,
        input=input,
    )
    if result.returncode != 0:
        return None
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return result.stdout.strip()


def git(*args: str, check: bool = True) -> str:
    result = subprocess.run(["git", *args], capture_output=True, text=True, check=check, cwd=ROOT)
    return result.stdout.strip()


def abort(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def confirm(prompt: str) -> bool:
    try:
        return input(f"{prompt} [Y/n] ").strip().lower() in ("", "y", "yes")
    except (KeyboardInterrupt, EOFError):
        print()
        return False


def parse_github_timestamp(timestamp: str) -> datetime:
    return datetime.fromisoformat(timestamp.replace("Z", "+00:00"))


# ── steps ────────────────────────────────────────────────────────────────────

def resolve_version() -> str:
    version = cargo_version()
    tag = f"v{version}"

    repo = gh("repo", "view", "--json", "nameWithOwner")["nameWithOwner"]
    exists = gh("api", f"repos/{repo}/git/ref/tags/{tag}") is not None

    if not exists:
        print(f"Using version {version} from Cargo.toml.")
        return version

    suggested = bump_minor(version)
    print(f"Tag {tag} already exists on GitHub.")
    if not confirm(f"Bump minor to {suggested} and release that?"):
        abort("aborted")

    subprocess.run(["python3", "scripts/bump_version.py", suggested], check=True, cwd=ROOT)
    subprocess.run(
        [str(Path(os.environ.get("HOME", "~")) / ".cargo" / "bin" / "cargo"), "generate-lockfile"],
        check=True,
        cwd=ROOT,
    )
    git("add", "Cargo.toml", "Cargo.lock")
    git("commit", "-m", f"chore: bump version to {suggested}")
    git("push")
    print(f"Pushed version bump to {suggested}.")
    return suggested


def wait_for_ci() -> None:
    sha = git("rev-parse", "HEAD")
    repo = gh("repo", "view", "--json", "nameWithOwner")["nameWithOwner"]

    print("Waiting for CI run to start", end="", flush=True)
    run_id = None
    for _ in range(36):  # up to 3 minutes
        runs = gh("run", "list", "--commit", sha, "--json", "databaseId,workflowName") or []
        ci_runs = [r for r in runs if "CI" in r.get("workflowName", "")]
        if ci_runs:
            run_id = ci_runs[0]["databaseId"]
            break
        print(".", end="", flush=True)
        time.sleep(5)
    print()

    if not run_id:
        abort("timed out waiting for CI run — check GitHub Actions manually")

    print(f"Watching CI run {run_id}...")
    result = subprocess.run(["gh", "run", "watch", str(run_id), "--exit-status"])
    if result.returncode != 0:
        abort("CI failed — fix the issues before releasing")


def wait_for_publish_workflow(started_after: datetime) -> None:
    print("Waiting for publish workflow to start", end="", flush=True)
    run_id = None
    repo = gh("repo", "view", "--json", "nameWithOwner")["nameWithOwner"]

    for _ in range(36):  # up to 3 minutes
        runs = gh(
            "run",
            "list",
            "--repo",
            repo,
            "--workflow",
            "publish.yml",
            "--limit",
            "20",
            "--json",
            "databaseId,workflowName,event,createdAt",
        ) or []
        publish_runs = [
            run for run in runs
            if run.get("workflowName") == "Publish"
            and run.get("event") == "release"
            and parse_github_timestamp(run["createdAt"]) >= started_after
        ]
        if publish_runs:
            run_id = publish_runs[0]["databaseId"]
            break
        print(".", end="", flush=True)
        time.sleep(5)
    print()

    if not run_id:
        abort("timed out waiting for publish workflow — check GitHub Actions manually")

    print(f"Watching publish workflow run {run_id}...")
    result = subprocess.run(["gh", "run", "watch", str(run_id), "--exit-status"], cwd=ROOT)
    if result.returncode != 0:
        abort("publish workflow failed — check GitHub Actions before announcing the release")


def get_release_notes(version: str) -> str:
    repo = gh("repo", "view", "--json", "nameWithOwner")["nameWithOwner"]
    tag = f"v{version}"
    data = gh("api", f"repos/{repo}/releases/generate-notes", "-f", f"tag_name={tag}")
    if not data or not isinstance(data, dict):
        return f"## ruffian {tag}\n\n"
    return data.get("body", "")


def edit_notes(initial: str) -> str:
    editor = os.environ.get("EDITOR") or ("notepad" if sys.platform == "win32" else "nano")
    with tempfile.NamedTemporaryFile(suffix=".md", mode="w", delete=False, encoding="utf-8") as f:
        f.write(initial)
        fname = f.name
    subprocess.run([editor, fname])
    notes = Path(fname).read_text(encoding="utf-8")
    Path(fname).unlink(missing_ok=True)
    return notes


def create_release(version: str, notes: str) -> None:
    tag = f"v{version}"
    subprocess.run(
        ["gh", "release", "create", tag,
         "--title", f"ruffian {tag}",
         "--notes", notes],
        check=True,
        cwd=ROOT,
    )
    print(f"\nRelease {tag} created — publish workflow is now running.")


# ── main ─────────────────────────────────────────────────────────────────────

def main() -> None:
    version = resolve_version()

    wait_for_ci()

    print("\nGenerating release notes...")
    notes = get_release_notes(version)

    print("\n── Auto-generated release notes ──────────────────────────────")
    print(notes)
    print("───────────────────────────────────────────────────────────────\n")

    if confirm("Open $EDITOR to review / rewrite the notes?"):
        notes = edit_notes(notes)

    release_started_after = datetime.now(timezone.utc) - timedelta(seconds=5)
    create_release(version, notes)
    wait_for_publish_workflow(release_started_after)


if __name__ == "__main__":
    main()
