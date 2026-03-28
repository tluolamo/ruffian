use crate::ruff::Violation;

/// Emit violations in ruff's concise text format:
///   path:row:col: CODE [*] message
///
/// The [*] marker is included when a fix is available, matching
/// `ruff check --output-format concise --quiet` exactly.
/// ruffian omits ruff's default rich format (source context, summary footer)
/// — use `--output-format json` if you need machine-readable output with full detail.
pub fn emit_text(violations: &[Violation]) {
    for v in violations {
        let fix_marker = if v.fix.is_some() { " [*]" } else { "" };
        println!(
            "{path}:{row}:{col}: {code}{fix_marker} {message}",
            path = v.filename,
            row = v.location.row,
            col = v.location.column,
            code = v.code,
            message = v.message,
        );
    }
}

pub fn emit_json(violations: &[Violation]) {
    println!(
        "{}",
        serde_json::to_string_pretty(violations).expect("serialization is infallible")
    );
}

/// Merge and sort violations from multiple sources by filename, then row, then column.
pub fn merge_sorted(mut violations: Vec<Violation>) -> Vec<Violation> {
    violations.sort_by(|a, b| {
        a.filename
            .cmp(&b.filename)
            .then(a.location.row.cmp(&b.location.row))
            .then(a.location.column.cmp(&b.location.column))
    });
    violations
}
