use std::process::ExitCode;

mod app;
mod cleanup;
mod cli;
mod config;
mod discovery;
mod manifest;
mod paths;
mod types;

fn main() -> ExitCode {
    match cli::parse() {
        Ok(options) => app::run(options),
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}
