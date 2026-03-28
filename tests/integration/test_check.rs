use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("ruffian").unwrap()
}

#[test]
fn check_clean_file_exits_zero() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("clean.py");
    fs::write(&file, "x = 1\n").unwrap();

    cmd()
        .args(["check", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn rfn001_fires_on_long_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("long.py");
    // Write a file with 1001 lines to trigger RFN001 (default 1000).
    let content: String = (1..=1001).map(|i| format!("x{i} = {i}\n")).collect();
    fs::write(&file, content).unwrap();

    // Write a minimal pyproject.toml enabling RFN001.
    let cfg = dir.path().join("pyproject.toml");
    fs::write(&cfg, "[tool.ruffian]\nselect = [\"RFN001\"]\n").unwrap();

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure(); // non-zero exit when violations found
}
