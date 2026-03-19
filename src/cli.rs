use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
#[command(
    name = "cargo-clean-global",
    bin_name = "cargo clean-global",
    version,
    about = "Safely cleans Cargo target directories across local Cargo projects.",
    long_about = None,
    after_help = "Cargo plugins can add new subcommands such as `cargo clean-global`.\nThey cannot extend Cargo's built-in `cargo clean` subcommand with a new `--global` flag."
)]
pub struct Cli {
    /// Show which target directories would be removed without deleting them.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the confirmation prompt and clean immediately.
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,

    /// Restrict scanning to a specific directory. Can be passed multiple times.
    #[arg(long = "root", value_name = "PATH", value_hint = ValueHint::DirPath)]
    pub roots: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn parses_roots_and_dry_run() {
        let options = Cli::try_parse_from([
            "cargo-clean-global",
            "--dry-run",
            "--yes",
            "--root",
            "workspace-a",
            "--root=workspace-b",
        ])
        .expect("arguments should parse");

        assert!(options.dry_run);
        assert!(options.yes);
        assert_eq!(
            options.roots,
            vec![PathBuf::from("workspace-a"), PathBuf::from("workspace-b")]
        );
    }

    #[test]
    fn rejects_unknown_arguments() {
        assert!(Cli::try_parse_from(["cargo-clean-global", "--unknown"]).is_err());
    }
}
