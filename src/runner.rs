use anyhow::Result;

use crate::{config, output, plugin, ruff, rules};

pub fn run_check(
    files: Vec<String>,
    output_format: String,
    fix: bool,
    select: Vec<String>,
    ignore: Vec<String>,
) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cfg = config::load(&cwd)?;

    // Merge CLI selects/ignores with config file values.
    let effective_select: Vec<String> = if select.is_empty() {
        cfg.select.clone()
    } else {
        select
    };
    let effective_ignore: Vec<String> = if ignore.is_empty() {
        cfg.ignore.clone()
    } else {
        ignore
    };

    // Run ruff, built-in rules, and plugins concurrently.
    // TODO(Phase 2): replace std::thread with tokio tasks once async is wired up.
    let files_clone = files.clone();
    let ruff_handle = std::thread::spawn(move || {
        let mut extra: Vec<&str> = vec![];
        if fix {
            extra.push("--fix");
        }
        ruff::check(&files_clone, &extra)
    });

    let files_clone2 = files.clone();
    let select_clone = effective_select.clone();
    let ignore_clone = effective_ignore.clone();
    let rules_config = cfg.rules.clone();
    let rules_handle = std::thread::spawn(move || {
        rules::run_all(&files_clone2, &select_clone, &ignore_clone, &rules_config)
    });

    let plugins = cfg.plugins.clone();
    let files_clone3 = files.clone();
    let version = env!("CARGO_PKG_VERSION");
    let plugins_handle = std::thread::spawn(move || -> Result<Vec<ruff::Violation>> {
        let mut all = vec![];
        for p in &plugins {
            match plugin::run_plugin(p, &files_clone3, version) {
                Ok(mut vs) => all.append(&mut vs),
                Err(e) => eprintln!("error: {e}"),
            }
        }
        Ok(all)
    });

    let mut violations = vec![];
    violations.append(&mut ruff_handle.join().expect("ruff thread panicked")?);
    violations.append(&mut rules_handle.join().expect("rules thread panicked")?);
    violations.append(&mut plugins_handle.join().expect("plugins thread panicked")?);

    let violations = output::merge_sorted(violations);
    let has_violations = !violations.is_empty();

    match output_format.as_str() {
        "json" => output::emit_json(&violations),
        _ => output::emit_text(&violations),
    }

    if has_violations {
        std::process::exit(1);
    }
    Ok(())
}
