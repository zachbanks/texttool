#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Extract Info
# @raycast.mode fullOutput

# Optional parameters:
# @raycast.icon 🔎
# @raycast.packageName Text Tools

# Documentation:
# @raycast.description Extract phone numbers, emails, dates, and more from the clipboard with tt. Shows the full Markdown result and copies it back to the clipboard. Runs on the system clipboard directly — no argument prompt.
# @raycast.author Zach Banks
# @raycast.authorURL https://github.com/zachbanks

set -euo pipefail

tt="$(command -v tt || echo "$HOME/.local/bin/tt")"
if [ ! -x "$tt" ]; then
  echo "tt not found — install from https://github.com/zachbanks/texttool"
  exit 1
fi

# No arguments: always read the system clipboard.
result="$(pbpaste | "$tt" extract)"

if [ -n "$result" ]; then
  printf '%s' "$result" | pbcopy
  printf '%s\n' "$result"
else
  echo "(no matches found)"
fi
