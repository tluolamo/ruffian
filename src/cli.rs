use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ruffian",
    about = "A superset of ruff with additional built-in rules and plugin support"
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run linting checks (ruff check + ruffian rules + plugins)
    Check {
        /// Files or directories to check
        #[arg(default_value = ".")]
        files: Vec<String>,

        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        output_format: String,

        /// Apply fixes where possible (ruff fixes only)
        #[arg(long)]
        fix: bool,

        /// Select specific rule codes
        #[arg(long, value_delimiter = ',')]
        select: Vec<String>,

        /// Ignore specific rule codes
        #[arg(long, value_delimiter = ',')]
        ignore: Vec<String>,
    },

    /// Format Python files (pure passthrough to ruff format)
    Format {
        /// Files or directories to format
        #[arg(default_value = ".")]
        files: Vec<String>,

        /// Check formatting without writing changes
        #[arg(long)]
        check: bool,
    },

    /// Show documentation for a rule
    Rule {
        /// Rule code (e.g. PLC0302)
        code: String,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Check {
            files,
            output_format,
            fix,
            select,
            ignore,
        } => crate::runner::run_check(files, output_format, fix, select, ignore),
        Command::Format { files, check } => crate::ruff::passthrough_format(files, check),
        Command::Rule { code } => crate::rules::print_rule_docs(&code),
    }
}
