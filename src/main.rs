use clap::Parser;
use std::process::ExitCode;

use cli::Cli;

mod app;
mod cleanup;
mod cli;
mod config;
mod discovery;
mod manifest;
mod paths;
mod types;

fn main() -> ExitCode {
    app::run(Cli::parse())
}
