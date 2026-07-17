#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Clean Text
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🧹
# @raycast.packageName Text Tools
# @raycast.argument1 { "type": "text", "placeholder": "text (optional — uses clipboard)", "optional": true }

# Documentation:
# @raycast.description Clean text with tt: normalize whitespace and line endings, capitalize acronyms, single letters, and sentence starts. Result is copied to the clipboard. Uses the clipboard as input when no text is given.
# @raycast.author Zach Banks
# @raycast.authorURL https://github.com/zachbanks

set -euo pipefail

tt="$(command -v tt || echo "$HOME/.local/bin/tt")"
if [ ! -x "$tt" ]; then
  echo "tt not found — install from https://github.com/zachbanks/texttool"
  exit 1
fi

# Use the argument if given, otherwise the clipboard.
if [ -n "${1:-}" ]; then
  input="$1"
else
  input="$(pbpaste)"
fi

result="$(printf '%s' "$input" | "$tt" clean --no-trailing-newline)"
printf '%s' "$result" | pbcopy
echo "$result"
