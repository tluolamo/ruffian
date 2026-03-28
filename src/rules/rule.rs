use crate::ruff::Violation;
use ruff_python_ast::ModModule;
use ruff_python_parser::Parsed;

/// Parsed representation of a Python source file passed to each rule.
#[allow(dead_code)] // `ast` is provided for rule authors; not all rules use it yet
pub struct ParsedFile {
    pub path: String,
    pub source: String,
    /// Parsed AST. `None` if the file contained syntax errors.
    pub ast: Option<Parsed<ModModule>>,
}

/// Every built-in rule implements this trait.
///
/// To add a rule:
/// 1. Create `src/rules/rfnXXX_<name>.rs` and implement `Rule`
/// 2. Register it in `src/rules/mod.rs` — one line, no other changes needed
pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn check(&self, file: &ParsedFile) -> Vec<Violation>;
}
