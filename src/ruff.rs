use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// A lint violation in ruff's JSON output format.
/// ruffian uses this same struct for all violation sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub code: String,
    pub message: String,
    pub filename: String,
    pub location: Location,
    pub end_location: Option<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub fix: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub row: u32,
    pub column: u32,
}

/// Run `ruff check --output-format json` on the given files and return parsed violations.
pub fn check(files: &[String], extra_args: &[&str]) -> Result<Vec<Violation>> {
    let output = Command::new("ruff")
        .arg("check")
        .arg("--output-format")
        .arg("json")
        .args(extra_args)
        .args(files)
        .output()
        .context("failed to spawn ruff — is it installed and on PATH?")?;

    if output.stdout.is_empty() {
        return Ok(vec![]);
    }
    let violations: Vec<Violation> = serde_json::from_slice(&output.stdout)
        .context("failed to parse ruff JSON output")?;
    Ok(violations)
}

/// Pass `ruff format` through directly, replacing the current process.
pub fn passthrough_format(files: Vec<String>, check: bool) -> Result<()> {
    let mut cmd = Command::new("ruff");
    cmd.arg("format");
    if check {
        cmd.arg("--check");
    }
    cmd.args(&files);

    // Replace current process with ruff so exit code propagates correctly.
    use std::os::unix::process::CommandExt;
    let err = cmd.exec();
    Err(anyhow::anyhow!("failed to exec ruff: {err}"))
}
