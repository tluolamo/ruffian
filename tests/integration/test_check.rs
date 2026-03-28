use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("ruffian").unwrap()
}

fn write_plugin(dir: &TempDir, name: &str, body: &str) -> String {
    let path = dir.path().join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path.to_str().unwrap().to_owned()
}

fn write_pyproject_with_plugin(dir: &TempDir, plugin_name: &str, executable: &str) {
    fs::write(
        dir.path().join("pyproject.toml"),
        format!(
            "[[tool.ruffian.plugins]]\nname = \"{plugin_name}\"\nexecutable = \"{executable}\"\n"
        ),
    )
    .unwrap();
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
fn plc0302_fires_on_long_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("long.py");
    let content: String = (1..=1001).map(|i| format!("x{i} = {i}\n")).collect();
    fs::write(&file, content).unwrap();

    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.ruffian]\nselect = [\"PLC0302\"]\n",
    )
    .unwrap();

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn plc0302_does_not_fire_at_exact_limit() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("exact.py");
    // Exactly 1000 lines — should not trigger.
    let content: String = (1..=1000).map(|i| format!("x{i} = {i}\n")).collect();
    fs::write(&file, content).unwrap();

    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.ruffian]\nselect = [\"PLC0302\"]\n",
    )
    .unwrap();

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn plc0302_respects_custom_max_lines_from_config() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("short.py");
    // 6 lines — only triggers if max-lines is set to 5.
    let content: String = (1..=6).map(|i| format!("x{i} = {i}\n")).collect();
    fs::write(&file, content).unwrap();

    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.ruffian]\nselect = [\"PLC0302\"]\n\n[tool.ruffian.rules.PLC0302]\nmax-lines = 5\n",
    )
    .unwrap();

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn rule_subcommand_prints_plc0302_docs() {
    cmd()
        .args(["rule", "PLC0302"])
        .assert()
        .success()
        .stdout(predicates::str::contains("PLC0302"))
        .stdout(predicates::str::contains("too-many-module-lines"));
}

#[test]
fn check_directory_with_no_python_files_exits_zero() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("notes.txt"), "hello\n").unwrap();

    cmd()
        .args(["check", dir.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn ruff_violations_are_included_in_output() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad.py");
    // F401 — unused import, caught by ruff's default rules.
    fs::write(&file, "import os\nx = 1\n").unwrap();

    cmd()
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn format_check_exits_zero_for_well_formatted_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("clean.py");
    fs::write(&file, "x = 1\n").unwrap();

    cmd()
        .args(["format", "--check", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn format_check_fails_for_unformatted_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("unformatted.py");
    fs::write(&file, "x=1\n").unwrap();

    cmd()
        .args(["format", "--check", file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn plugin_with_bad_executable_does_not_count_as_lint_failure() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("a.py");
    fs::write(&file, "x = 1\n").unwrap();

    write_pyproject_with_plugin(&dir, "bad-plugin", "/nonexistent/plugin.sh");

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn plugin_returning_violation_causes_failure() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("a.py");
    fs::write(&file, "x = 1\n").unwrap();

    let violation = format!(
        r#"[{{"code":"RFC001","message":"test violation","filename":"{path}","location":{{"row":1,"column":0}},"end_location":null,"url":null,"fix":null}}]"#,
        path = file.display()
    );
    let plugin = write_plugin(&dir, "plugin.sh", &format!("echo '{violation}'"));
    write_pyproject_with_plugin(&dir, "test-plugin", &plugin);

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .failure();
}

#[test]
fn plugin_returning_empty_array_exits_zero() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("a.py");
    fs::write(&file, "x = 1\n").unwrap();

    let plugin = write_plugin(&dir, "plugin.sh", "echo '[]'");
    write_pyproject_with_plugin(&dir, "test-plugin", &plugin);

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn plugin_stderr_is_forwarded() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("a.py");
    fs::write(&file, "x = 1\n").unwrap();

    let plugin = write_plugin(&dir, "plugin.sh", "echo 'oops' >&2\necho '[]'");
    write_pyproject_with_plugin(&dir, "test-plugin", &plugin);

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicates::str::contains("[plugin: test-plugin] oops"));
}

#[test]
fn plugin_nonzero_exit_does_not_count_as_lint_failure() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("a.py");
    fs::write(&file, "x = 1\n").unwrap();

    // Plugin error is reported on stderr but does not produce violations,
    // so ruffian exits 0 (no lint failures).
    let plugin = write_plugin(&dir, "plugin.sh", "exit 1");
    write_pyproject_with_plugin(&dir, "test-plugin", &plugin);

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn json_output_is_valid_json_array() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad.py");
    fs::write(&file, "import os\nx = 1\n").unwrap();

    let output = cmd()
        .args(["check", "--output-format", "json", file.to_str().unwrap()])
        .output()
        .unwrap();

    let parsed: Value = serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    assert!(parsed.is_array(), "JSON output must be an array");
}

#[test]
fn json_output_contains_required_violation_fields() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("bad.py");
    fs::write(&file, "import os\nx = 1\n").unwrap();

    let output = cmd()
        .args(["check", "--output-format", "json", file.to_str().unwrap()])
        .output()
        .unwrap();

    let violations: Vec<Value> =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    assert!(!violations.is_empty(), "expected at least one violation");

    let v = &violations[0];
    assert!(v["code"].is_string(), "violation must have a string 'code'");
    assert!(
        v["message"].is_string(),
        "violation must have a string 'message'"
    );
    assert!(
        v["filename"].is_string(),
        "violation must have a string 'filename'"
    );
    assert!(
        v["location"]["row"].is_number(),
        "violation must have a numeric 'location.row'"
    );
    assert!(
        v["location"]["column"].is_number(),
        "violation must have a numeric 'location.column'"
    );
}

#[test]
fn json_output_includes_ruffian_violations() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("long.py");
    let content: String = (1..=1001).map(|i| format!("x{i} = {i}\n")).collect();
    fs::write(&file, content).unwrap();

    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.ruffian]\nselect = [\"PLC0302\"]\n",
    )
    .unwrap();

    let output = cmd()
        .current_dir(&dir)
        .args(["check", "--output-format", "json", file.to_str().unwrap()])
        .output()
        .unwrap();

    let violations: Vec<Value> =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");

    let has_plc0302 = violations
        .iter()
        .any(|v| v["code"].as_str() == Some("PLC0302"));
    assert!(has_plc0302, "expected PLC0302 violation in JSON output");
}

#[test]
fn json_output_is_empty_array_for_clean_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("clean.py");
    fs::write(&file, "x = 1\n").unwrap();

    let output = cmd()
        .args([
            "check",
            "--output-format",
            "json",
            "--ignore",
            "PLC0302",
            file.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let violations: Vec<Value> =
        serde_json::from_slice(&output.stdout).expect("stdout must be valid JSON");
    assert!(violations.is_empty(), "expected empty array for clean file");
}

#[test]
fn fix_flag_applies_ruff_fixes_and_exits_zero() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fixable.py");
    // F401 — unused import; ruff can auto-fix this.
    fs::write(&file, "import os\nx = 1\n").unwrap();

    cmd()
        .args(["check", "--fix", file.to_str().unwrap()])
        .assert()
        .success();

    let fixed = fs::read_to_string(&file).unwrap();
    assert!(
        !fixed.contains("import os"),
        "expected ruff --fix to remove unused import, got: {fixed}"
    );
}

#[test]
fn plugin_invalid_json_does_not_count_as_lint_failure() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("a.py");
    fs::write(&file, "x = 1\n").unwrap();

    let plugin = write_plugin(&dir, "plugin.sh", "echo 'not json'");
    write_pyproject_with_plugin(&dir, "test-plugin", &plugin);

    cmd()
        .current_dir(&dir)
        .args(["check", file.to_str().unwrap()])
        .assert()
        .success();
}
