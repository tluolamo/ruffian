#![cfg(unix)]

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn fake_ruff(dir: &TempDir, body: &str) {
    let path = dir.path().join("ruff");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn check(dir: &TempDir) -> Command {
    let file = dir.path().join("clean.py");
    fs::write(&file, "x = 1\n").unwrap();
    let mut cmd = Command::cargo_bin("ruffian").unwrap();
    cmd.current_dir(dir)
        .env("PATH", dir.path())
        .args(["check", file.to_str().unwrap()]);
    cmd
}

#[test]
fn accepts_successful_json_output() {
    let dir = TempDir::new().unwrap();
    fake_ruff(&dir, "echo '[]'");
    check(&dir).assert().success();
}

#[test]
fn accepts_lint_violation_exit_code() {
    let dir = TempDir::new().unwrap();
    fake_ruff(&dir, "echo '[{\"code\":\"F401\",\"message\":\"unused\",\"filename\":\"a.py\",\"location\":{\"row\":1,\"column\":1},\"end_location\":null,\"fix\":null}]'\nexit 1");
    check(&dir).assert().code(1).stdout(contains("F401"));
}

#[test]
fn reports_ruff_failure_even_with_json_output() {
    let dir = TempDir::new().unwrap();
    fake_ruff(&dir, "echo '[]'\necho 'unsupported option' >&2\nexit 2");
    check(&dir)
        .assert()
        .failure()
        .stderr(contains("ruff check failed"))
        .stderr(contains("unsupported option"));
}

#[test]
fn reports_ruff_failure_with_empty_output() {
    let dir = TempDir::new().unwrap();
    fake_ruff(&dir, "echo 'invalid config' >&2\nexit 2");
    check(&dir)
        .assert()
        .failure()
        .stderr(contains("invalid config"));
}

#[test]
fn rejects_empty_success_output() {
    let dir = TempDir::new().unwrap();
    fake_ruff(&dir, "exit 0");
    check(&dir)
        .assert()
        .failure()
        .stderr(contains("failed to parse ruff JSON output"));
}

#[test]
fn forwards_ruff_warnings() {
    let dir = TempDir::new().unwrap();
    fake_ruff(&dir, "echo 'ruff warning' >&2\necho '[]'");
    check(&dir)
        .assert()
        .success()
        .stderr(contains("ruff warning"));
}
