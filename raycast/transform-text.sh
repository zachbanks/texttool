#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Transform Text
# @raycast.mode compact

# Optional parameters:
# @raycast.icon 🧰
# @raycast.packageName Text Tools
# @raycast.argument1 { "type": "dropdown", "placeholder": "Operation", "data": [{"title": "Clean", "value": "clean"}, {"title": "Title Case", "value": "titlecase"}, {"title": "Strip Edges", "value": "strip"}, {"title": "Extract Info", "value": "extract"}, {"title": "Slug", "value": "slug"}, {"title": "Unslug", "value": "unslug"}, {"title": "Squeeze", "value": "squeeze"}, {"title": "UPPERCASE", "value": "upper"}, {"title": "lowercase", "value": "lower"}, {"title": "camelCase", "value": "camel"}, {"title": "PascalCase", "value": "pascal"}, {"title": "snake_case", "value": "snake"}, {"title": "kebab-case", "value": "kebab"}, {"title": "CONSTANT_CASE", "value": "constant"}, {"title": "mOcKiNg", "value": "mock"}] }
# @raycast.argument2 { "type": "text", "placeholder": "text (optional — uses clipboard)", "optional": true }

# Documentation:
# @raycast.description Run any tt text transform, chosen from a dropdown, on the clipboard (or on text you type). The result is copied back to the clipboard.
# @raycast.author Zach Banks
# @raycast.authorURL https://github.com/zachbanks

set -euo pipefail

tt="$(command -v tt || echo "$HOME/.local/bin/tt")"
if [ ! -x "$tt" ]; then
  echo "tt not found — install from https://github.com/zachbanks/texttool"
  exit 1
fi

op="$1"

# Use the second argument if given, otherwise the clipboard.
if [ -n "${2:-}" ]; then
  input="$2"
else
  input="$(pbpaste)"
fi

# Command substitution strips any trailing newline, keeping the clipboard clean.
result="$(printf '%s' "$input" | "$tt" "$op")"
printf '%s' "$result" | pbcopy
echo "$result"
