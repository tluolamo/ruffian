use crate::config::PluginConfig;
use crate::ruff::Violation;
use anyhow::{Context, Result};
use serde_json;
use std::io::Write as _;
use std::process::{Command, Stdio};

/// Invoke a single plugin executable and return its violations.
pub fn run_plugin(plugin: &PluginConfig, files: &[String], ruffian_version: &str) -> Result<Vec<Violation>> {
    let stdin_payload = serde_json::json!({
        "ruffian_version": ruffian_version,
        "config": plugin.config,
    });

    let mut child = Command::new(&plugin.executable)
        .args(files)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn plugin '{}'", plugin.name))?;

    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(stdin_payload.to_string().as_bytes())
        .with_context(|| format!("failed to write stdin to plugin '{}'", plugin.name))?;

    let output = child
        .wait_with_output()
        .with_context(|| format!("failed to wait for plugin '{}'", plugin.name))?;

    // Forward plugin stderr with a prefix.
    if !output.stderr.is_empty() {
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            eprintln!("[plugin: {}] {}", plugin.name, line);
        }
    }

    if !output.status.success() {
        anyhow::bail!(
            "plugin '{}' exited with status {}",
            plugin.name,
            output.status
        );
    }

    if output.stdout.is_empty() {
        return Ok(vec![]);
    }
    let violations: Vec<Violation> = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("plugin '{}' produced invalid JSON", plugin.name))?;
    Ok(violations)
}
