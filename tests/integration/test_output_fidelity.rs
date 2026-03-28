/// Output fidelity tests — verify that `ruffian check` produces output identical
/// to `ruff check` when no ruffian-specific rules are enabled.
///
/// This is the Phase 1 milestone: `ruffian check .` and `ruff check .` must be
/// indistinguishable on a vanilla project.
use assert_cmd::Command;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn ruffian() -> Command {
    Command::cargo_bin("ruffian").unwrap()
}

fn ruff_output(args: &[&str]) -> (Vec<u8>, Vec<u8>) {
    let out = StdCommand::new("ruff")
        .args(args)
        .output()
        .expect("ruff must be on PATH");
    (out.stdout, out.stderr)
}

/// Run ruffian and ruff on the same file with no ruffian rules enabled and
/// assert their stdout is identical.
///
/// ruffian's text output matches ruff's `--output-format concise`:
///   path:row:col: CODE message
/// (no source context, no summary footer)
fn assert_same_output(file: &str) {
    let ruffian_out = ruffian()
        .args(["check", "--ignore", "PLC0302", file])
        .output()
        .unwrap();

    let (ruff_stdout, _) = ruff_output(&["check", "--output-format", "concise", "--quiet", file]);

    assert_eq!(
        String::from_utf8_lossy(&ruffian_out.stdout),
        String::from_utf8_lossy(&ruff_stdout),
        "ruffian and ruff produced different output for {file}"
    );
}

#[test]
fn output_matches_ruff_for_clean_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("clean.py");
    fs::write(&file, "x = 1\n").unwrap();
    assert_same_output(file.to_str().unwrap());
}

#[test]
fn output_matches_ruff_for_file_with_violations() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad.py");
    // F401 — unused import, reliably caught by ruff's default config.
    fs::write(&file, "import os\nx = 1\n").unwrap();
    assert_same_output(file.to_str().unwrap());
}
