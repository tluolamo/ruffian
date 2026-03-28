use crate::ruff::{Location, Violation};
use crate::rules::rule::{ParsedFile, Rule};

const DEFAULT_MAX_LINES: usize = 1000;

pub struct TooManyModuleLines {
    pub max_lines: usize,
}

impl Default for TooManyModuleLines {
    fn default() -> Self {
        Self { max_lines: DEFAULT_MAX_LINES }
    }
}

impl Rule for TooManyModuleLines {
    fn code(&self) -> &'static str {
        "RFN001"
    }

    fn name(&self) -> &'static str {
        "too-many-module-lines"
    }

    fn description(&self) -> &'static str {
        "Module exceeds the maximum number of lines."
    }

    fn check(&self, file: &ParsedFile) -> Vec<Violation> {
        let line_count = file.source.lines().count();
        if line_count <= self.max_lines {
            return vec![];
        }
        vec![Violation {
            code: self.code().to_owned(),
            message: format!(
                "Module has {line_count} lines, exceeding the maximum of {}.",
                self.max_lines
            ),
            filename: file.path.clone(),
            location: Location { row: 1, column: 0 },
            end_location: None,
            url: None,
            fix: None,
        }]
    }
}
