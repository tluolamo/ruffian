use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod noqa;
mod output;
mod plugin;
mod ruff;
mod rules;
mod runner;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli::run(cli)
}
