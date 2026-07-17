#!/usr/bin/env bash
#
# Cut a minor or major release at an exact version.
#
# The pre-commit hook auto-bumps the PATCH component on every normal commit, so
# to land an exact version (e.g. a 0.2.0 minor or 1.0.0 major) this script sets
# the version explicitly and commits with --no-verify to bypass that bump, then
# tags the commit `vX.Y.Z`.
#
# Usage: scripts/release.sh <MAJOR.MINOR.PATCH>
set -euo pipefail

version="${1:-}"
if ! [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: scripts/release.sh <MAJOR.MINOR.PATCH>   (e.g. 0.2.0)" >&2
  exit 1
fi

cd "$(git rev-parse --show-toplevel)"

if [ -n "$(git status --porcelain)" ]; then
  echo "release: working tree is not clean; commit or stash changes first" >&2
  exit 1
fi

if git rev-parse -q --verify "refs/tags/v${version}" >/dev/null; then
  echo "release: tag v${version} already exists" >&2
  exit 1
fi

# Set the exact version in Cargo.toml (top-level package version line only).
tmp="$(mktemp)"
awk -v nv="${version}" '
  !done && /^version = "/ { sub(/"[^"]*"/, "\"" nv "\""); done = 1 }
  { print }
' Cargo.toml >"${tmp}"
mv "${tmp}" Cargo.toml

cargo update --workspace --offline >/dev/null 2>&1 \
  || cargo update --workspace >/dev/null 2>&1 \
  || true

git add Cargo.toml Cargo.lock

# --no-verify skips the pre-commit patch bump so the version lands exactly.
git commit --no-verify -m "Release v${version}"
git tag -a "v${version}" -m "v${version}"
echo "release: committed and tagged v${version}"

# Publish: push the branch + tag, then cut a GitHub Release with generated notes.
branch="$(git rev-parse --abbrev-ref HEAD)"
if git remote get-url origin >/dev/null 2>&1; then
  git push origin "${branch}" --tags
  echo "release: pushed ${branch} and tag v${version}"

  if command -v gh >/dev/null 2>&1; then
    gh release create "v${version}" \
      --title "v${version}" \
      --generate-notes \
      --verify-tag \
      --latest
    echo "release: created GitHub Release v${version}"
  else
    echo "release: gh not found; create the GitHub Release manually" >&2
  fi
else
  echo "release: no 'origin' remote; skipped push and GitHub Release" >&2
fi
