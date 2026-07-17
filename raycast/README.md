# Raycast script commands for `tt`

Two [Raycast script commands](https://github.com/raycast/script-commands) that
run [`tt`](../README.md) on the clipboard (or on text you type):

| Command      | Runs             | Icon |
|--------------|------------------|------|
| `Clean Text` | `tt clean`       | 🧹   |
| `Title Case` | `tt titlecase`   | 🔠   |

Both take an **optional** text argument. With no argument they read the
clipboard (`pbpaste`); either way the result is copied back to the clipboard
(`pbcopy`) and shown in Raycast.

## Setup

1. Install `tt` (see the [main README](../README.md)) so it is on your `PATH`
   or at `~/.local/bin/tt`.
2. In Raycast: **Settings → Extensions → Script Commands → Add Script
   Directory**, and choose this `raycast/` folder.
3. Search Raycast for **Clean Text** or **Title Case**.

The scripts are already executable; if you copy them elsewhere, run
`chmod +x *.sh`.
