# cargo-clean-global

`cargo-clean-global` is a Cargo plugin that finds Cargo projects on your machine and cleans their build output directories in one command.

It is designed as the practical equivalent of a global Cargo clean operation. Because Cargo plugins cannot extend the built-in `cargo clean` command with a `--global` flag, this project is used as:

```bash
cargo clean-global
```

Direct execution via `cargo-clean-global` is intentionally rejected. Use the Cargo subcommand form only.

## Installation

```bash
cargo install cargo-clean-global
```

Install from a local checkout during development:

```bash
cargo install --path .
```

## Usage

Clean all discovered Cargo project build directories:

```bash
cargo clean-global
```

Skip the confirmation prompt and clean immediately:

```bash
cargo clean-global --yes
```

Preview what would be cleaned without deleting anything:

```bash
cargo clean-global --dry-run
```

Restrict scanning to a specific directory:

```bash
cargo clean-global --root ~/projects
```

Restrict scanning to multiple directories:

```bash
cargo clean-global --root ~/projects --root /work/rust
```

Show help:

```bash
cargo clean-global --help
```

## Contribution

Contributions of any kind are welcome! If you have ideas or suggestions, feel free to open an issue or submit a pull request.

## Release Automation

Pushing a version tag that matches `v<package-version>` runs the release workflow, builds the release artifacts, creates the GitHub Release, and publishes the crate to crates.io.

Repository maintainers must configure the `CRATES_IO_TOKEN` GitHub Actions secret before using this flow.

## License

MIT License © 2026 [Clover You](https://github.com/clovu)
