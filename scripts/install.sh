#!/usr/bin/env bash
#
# Build a release binary and install it to ~/.local/bin.
#
# This is the same action the post-commit hook performs, exposed as a manual
# command for when you want to (re)install without making a commit.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

version="$(awk -F'"' '/^version = "/ {print $2; exit}' Cargo.toml)"
echo "Building tt ${version} (release)…"
cargo build --release

dest="${HOME}/.local/bin"
mkdir -p "${dest}"
install -m 0755 target/release/tt "${dest}/tt"
echo "Installed tt ${version} -> ${dest}/tt"
