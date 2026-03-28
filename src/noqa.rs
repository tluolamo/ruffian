use crate::ruff::Violation;
use std::collections::HashMap;

/// Filter violations suppressed by `# ruffian: noqa` comments in source files.
///
/// Supports:
///   `# ruffian: noqa`            — suppress all ruffian violations on this line
///   `# ruffian: noqa CODE`       — suppress a specific rule code
///   `# ruffian: noqa CODE1, CODE2` — suppress multiple codes
///
/// Note: ruff's own `# noqa` handling is done by ruff itself before it reports violations.
/// This filter applies to ruffian built-in rules and plugin violations.
pub fn filter_noqa(violations: Vec<Violation>) -> Vec<Violation> {
    let mut lines_cache: HashMap<String, Vec<String>> = HashMap::new();

    violations
        .into_iter()
        .filter(|v| {
            let lines = lines_cache.entry(v.filename.clone()).or_insert_with(|| {
                std::fs::read_to_string(&v.filename)
                    .map(|s| s.lines().map(str::to_owned).collect())
                    .unwrap_or_default()
            });
            let row = v.location.row as usize;
            if row == 0 || row > lines.len() {
                return true;
            }
            !is_suppressed(&lines[row - 1], &v.code)
        })
        .collect()
}

fn is_suppressed(line: &str, code: &str) -> bool {
    let Some(pos) = line.find("# ruffian: noqa") else {
        return false;
    };
    let after = line[pos + "# ruffian: noqa".len()..].trim_start();
    // Bare `# ruffian: noqa` — suppress all ruffian violations on this line.
    if after.is_empty() {
        return true;
    }
    // `# ruffian: noqa CODE1, CODE2` — suppress specific codes.
    after
        .split(',')
        .map(str::trim)
        .any(|c| c.eq_ignore_ascii_case(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_noqa_suppresses_any_code() {
        assert!(is_suppressed("x = 1  # ruffian: noqa", "PLC0302"));
    }

    #[test]
    fn specific_code_suppresses_matching_code() {
        assert!(is_suppressed("x = 1  # ruffian: noqa PLC0302", "PLC0302"));
    }

    #[test]
    fn specific_code_does_not_suppress_other_code() {
        assert!(!is_suppressed("x = 1  # ruffian: noqa RFN001", "PLC0302"));
    }

    #[test]
    fn multiple_codes_suppresses_any_listed() {
        assert!(is_suppressed(
            "x = 1  # ruffian: noqa RFN001, PLC0302",
            "PLC0302"
        ));
    }

    #[test]
    fn no_noqa_comment_does_not_suppress() {
        assert!(!is_suppressed("x = 1  # some other comment", "PLC0302"));
    }

    #[test]
    fn noqa_is_case_insensitive_for_codes() {
        assert!(is_suppressed("x = 1  # ruffian: noqa plc0302", "PLC0302"));
    }
}
