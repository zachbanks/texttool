#!/usr/bin/env bash
#
# Enable the tracked git hooks for this repository.
#
# The hooks live in .githooks/ (version controlled) rather than .git/hooks/ (not
# version controlled), so they must be activated once per clone by pointing
# core.hooksPath at them.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit .githooks/post-commit

echo "Enabled git hooks (core.hooksPath=.githooks):"
echo "  pre-commit  -> auto-bump patch version"
echo "  post-commit -> build release + install to ~/.local/bin"
