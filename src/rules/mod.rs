mod rule;
mod too_many_module_lines;

pub use rule::{ParsedFile, Rule};

use crate::ruff::Violation;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;

// ── Rule registry ────────────────────────────────────────────────────────────
// Add one line here for each new rule. No other changes required.
fn all_rules(rules_config: &HashMap<String, toml::Value>) -> Vec<Box<dyn Rule>> {
    vec![Box::new(
        too_many_module_lines::TooManyModuleLines::from_config(rules_config.get("PLC0302")),
    )]
}
// ─────────────────────────────────────────────────────────────────────────────

/// Run all enabled rules against the given files and return violations.
pub fn run_all(
    files: &[String],
    select: &[String],
    ignore: &[String],
    rules_config: &HashMap<String, toml::Value>,
) -> Result<Vec<Violation>> {
    let rules = all_rules(rules_config);
    let active_rules: Vec<&Box<dyn Rule>> = rules
        .iter()
        .filter(|r| is_active(r.code(), select, ignore))
        .collect();

    let violations: Vec<Violation> = files
        .par_iter()
        .flat_map(|path| {
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("warning: could not read {path}: {e}");
                    return vec![];
                }
            };
            let parsed = ParsedFile {
                path: path.clone(),
                source,
            };
            active_rules
                .iter()
                .flat_map(|rule| rule.check(&parsed))
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(violations)
}

/// Print documentation for a single rule code to stdout.
pub fn print_rule_docs(code: &str) -> Result<()> {
    let rules = all_rules(&HashMap::new());
    match rules.iter().find(|r| r.code().eq_ignore_ascii_case(code)) {
        Some(rule) => {
            println!("{} — {}", rule.code(), rule.name());
            println!();
            println!("{}", rule.description());
            Ok(())
        }
        None => {
            eprintln!("No rule found for code: {code}");
            std::process::exit(1);
        }
    }
}

fn is_active(code: &str, select: &[String], ignore: &[String]) -> bool {
    if ignore.iter().any(|c| c.eq_ignore_ascii_case(code)) {
        return false;
    }
    if select.is_empty() {
        return true;
    }
    select.iter().any(|c| c.eq_ignore_ascii_case(code))
}
