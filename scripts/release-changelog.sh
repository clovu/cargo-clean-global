#!/usr/bin/env sh
set -eu

tag="v${NEW_VERSION:?NEW_VERSION is required}"
workspace="${WORKSPACE_ROOT:-$(pwd)}"
output="${workspace}/CHANGELOG.md"

if [ "${DRY_RUN:-false}" = "true" ]; then
  preview="${TMPDIR:-/tmp}/cargo-clean-global-${tag}-CHANGELOG.md"
  git -C "${workspace}" cliff --tag "${tag}" --output "${preview}"
  echo "dry-run changelog preview written to ${preview}"
else
  git -C "${workspace}" cliff --tag "${tag}" --output "${output}"
fi
