set dotenv-load := true

default:
    @just --list

check-tools:
    @command -v cargo-release >/dev/null || { echo "missing cargo-release: cargo install cargo-release"; exit 1; }
    @command -v git-cliff >/dev/null || { echo "missing git-cliff: cargo install git-cliff"; exit 1; }

fmt:
    cargo fmt --all -- --check

check:
    cargo check --workspace --all-targets --locked

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features --locked

ci: fmt check clippy test

changelog version:
    git cliff --tag v{{version}} --output CHANGELOG.md

release-notes:
    git cliff --unreleased

release-dry-run version: check-tools
    cargo release {{version}} --no-publish --no-push

release version: check-tools ci
    cargo release {{version}} --no-publish --no-push --execute

release-execute version: check-tools ci
    cargo release {{version}} --no-publish --execute
