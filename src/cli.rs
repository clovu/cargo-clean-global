use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;
use clap::ValueHint;

const SUBCOMMAND_NAME: &str = "clean-global";
const DIRECT_INVOCATION_MESSAGE: &str = "run this tool as `cargo clean-global`; direct `cargo-clean-global` invocation is not supported";

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

pub(crate) fn parse() -> Result<Cli, &'static str> {
    let args = normalize_args(std::env::args_os(), std::env::var_os("CARGO").is_some())?;
    Ok(Cli::parse_from(args))
}

fn normalize_args<I, T>(args: I, invoked_via_cargo: bool) -> Result<Vec<OsString>, &'static str>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut args: Vec<OsString> = args.into_iter().map(Into::into).collect();

    if invoked_via_cargo && has_cargo_forwarded_subcommand(&args) {
        args.remove(1);
        return Ok(args);
    }

    if cfg!(debug_assertions) {
        // In debug mode, allow direct invocation for easier local testing.
        // Still reject spoofed forwarded subcommand names when not invoked by cargo.
        if has_cargo_forwarded_subcommand(&args) {
            return Err(DIRECT_INVOCATION_MESSAGE);
        }

        return Ok(args);
    }

    Err(DIRECT_INVOCATION_MESSAGE)
}

fn has_cargo_forwarded_subcommand(args: &[OsString]) -> bool {
    if args.len() < 2 {
        return false;
    }

    let Some(subcommand_name) = args[1].to_str() else {
        return false;
    };

    subcommand_name == SUBCOMMAND_NAME
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use super::normalize_args;
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

    #[test]
    fn strips_forwarded_hyphenated_subcommand_name() {
        let args = normalize_args(["cargo-clean-global", "clean-global", "--dry-run"], true)
            .expect("cargo should normalize the forwarded subcommand name");
        let options = Cli::try_parse_from(args)
            .expect("cargo should be able to forward the hyphenated subcommand name");

        assert!(options.dry_run);
    }

    #[test]
    fn rejects_direct_binary_invocation() {
        if cfg!(debug_assertions) {
            assert!(normalize_args(["cargo-clean-global", "--dry-run"], false).is_ok());
        } else {
            assert!(normalize_args(["cargo-clean-global", "--dry-run"], false).is_err());
        }
    }

    #[test]
    fn rejects_spoofed_subcommand_name_without_cargo_env() {
        assert!(
            normalize_args(["cargo-clean-global", "clean-global", "--dry-run",], false).is_err()
        );
    }
}
