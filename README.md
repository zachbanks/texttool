# texttool

A unified, extensible command-line toolkit for text manipulation, written in
Rust. It merges what used to be separate scripts (`textclean`, `smarttitlecase`,
…) into one binary with a consistent, colorful interface, and is designed so new
text operations can be added with a few lines of code.

## Highlights

- **One binary, many operations** — each operation is a subcommand.
- **Consistent I/O** — every subcommand reads from file operands or standard
  input and writes to standard output (or `--output <FILE>`).
- **Colored, discoverable help** — `texttool --help` and
  `texttool <op> --help` are colorized and example-driven.
- **Extensible by design** — operations implement a single `Transform` trait and
  register themselves; see [Extending](#extending).

## Install

```sh
# One-time: build and copy the binary to ~/.local/bin
make install        # or: ./scripts/install.sh
```

Make sure `~/.local/bin` is on your `PATH`.

### Auto-build on every commit (optional, for contributors)

This repo ships git hooks that, on every commit, **bump the patch version** and
**rebuild + reinstall** the binary to `~/.local/bin`. Enable them once per clone:

```sh
make setup          # or: ./scripts/setup-hooks.sh
```

## Usage

```
texttool <OPERATION> [FILES]... [OPTIONS]
```

If no files are given, input is read from standard input. Output goes to
standard output unless `-o/--output <FILE>` is given.

### Operations

| Operation   | Aliases      | Description                                         |
|-------------|--------------|-----------------------------------------------------|
| `clean`     |              | Tidy whitespace, line endings, invisible characters |
| `titlecase` | `title`, `tc`| Convert text to smart Title Case                    |
| `upper`     | `uc`         | Convert text to UPPERCASE                            |
| `lower`     | `lc`         | Convert text to lowercase                            |

More operations (`slug`, …) are added in subsequent commits — run
`texttool --help` for the current list.

#### `titlecase`

Smart title casing: minor words (`a`, `an`, `the`, `of`, `to`, …) stay lowercase
unless they are the first/last word or begin a subtitle after a colon; acronyms
and brand names with internal capitals (`NASA`, `iPhone`) are preserved;
hyphenated compounds are capitalized part-by-part; spacing and line breaks are
kept intact.

```sh
echo 'the quick brown fox: a tale of two-cities' | texttool titlecase
# The Quick Brown Fox: A Tale of Two-Cities
```

#### `clean`

Normalizes line endings to LF, applies Unicode NFC, strips control/zero-width
characters, removes trailing whitespace, squeezes repeated spaces (preserving
indentation), collapses runs of blank lines, and ends with a single newline.

| Flag                    | Effect                                            |
|-------------------------|---------------------------------------------------|
| `--ascii`               | Fold smart quotes/dashes/ellipses to ASCII        |
| `--no-squeeze`          | Keep repeated spaces                              |
| `--keep-blank-lines`    | Keep consecutive blank lines                      |
| `--no-trailing-newline` | Do not force a trailing newline                  |

### Examples

```sh
# Uppercase a file
texttool upper notes.txt

# Lowercase from a pipe
echo 'HELLO' | texttool lower

# Write the result to a file
texttool upper input.txt -o SHOUTING.txt
```

## Extending

Every operation is a type implementing the `Transform` trait
(`src/transform.rs`):

```rust
impl Transform for MyOp {
    fn name(&self) -> &'static str { "myop" }
    fn about(&self) -> &'static str { "Describe what it does" }
    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(/* transformed text */)
    }
}
```

Then add one line to `default_registry()` in `src/registry.rs`:

```rust
registry.register(MyOp);
```

The subcommand, its colored help, and its I/O wiring are generated
automatically. Transforms that need their own flags implement `augment()` to add
them and read them back from `args` in `apply()`.

## Development

```sh
make check          # fmt-check + clippy (deny warnings) + tests
make fmt            # format
make test           # run tests
```

## License

[MIT](LICENSE) © Zach Banks
