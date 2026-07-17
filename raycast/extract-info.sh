#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Extract Info
# @raycast.mode fullOutput

# Optional parameters:
# @raycast.icon 🔎
# @raycast.packageName Text Tools
# @raycast.argument1 { "type": "text", "placeholder": "text (optional — uses clipboard)", "optional": true }
# @raycast.argument2 { "type": "text", "placeholder": "only: emails,phones (optional)", "optional": true }

# Documentation:
# @raycast.description Extract phone numbers, emails, dates, and more from text with tt. Shows the full Markdown result and copies it to the clipboard. Uses the clipboard as input when no text is given; the second argument optionally limits categories.
# @raycast.author Zach Banks
# @raycast.authorURL https://github.com/zachbanks

set -euo pipefail

tt="$(command -v tt || echo "$HOME/.local/bin/tt")"
if [ ! -x "$tt" ]; then
  echo "tt not found — install from https://github.com/zachbanks/texttool"
  exit 1
fi

# Input: the first argument if given, otherwise the clipboard.
if [ -n "${1:-}" ]; then
  input="$1"
else
  input="$(pbpaste)"
fi

# Optional category filter via the second argument.
if [ -n "${2:-}" ]; then
  result="$(printf '%s' "$input" | "$tt" extract --only "$2")"
else
  result="$(printf '%s' "$input" | "$tt" extract)"
fi

if [ -n "$result" ]; then
  printf '%s' "$result" | pbcopy
  printf '%s\n' "$result"
else
  echo "(no matches found)"
fi
