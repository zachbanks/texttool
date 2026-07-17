# texttool

A unified, extensible command-line toolkit for text manipulation, written in
Rust. It merges what used to be separate scripts (`textclean`, `smarttitlecase`,
…) into one binary — installed as `tt` — with a consistent, colorful interface,
and is designed so new text operations can be added with a few lines of code.

## Highlights

- **One binary, many operations** — each operation is a subcommand.
- **Consistent I/O** — every subcommand reads from file operands or standard
  input and writes to standard output (or `--output <FILE>`).
- **Colored, discoverable help** — `tt --help` and
  `tt <op> --help` are colorized and example-driven.
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
tt <OPERATION> [FILES]... [OPTIONS]
```

If no files are given, input is read from standard input. Output goes to
standard output unless `-o/--output <FILE>` is given.

### Operations

| Operation   | Aliases                | Description                                      |
|-------------|------------------------|--------------------------------------------------|
| `clean`     |                        | Tidy whitespace, line endings, invisible chars   |
| `squeeze`   | `sq`, `normalize-ws`   | Collapse excess spaces, tabs, and newlines       |
| `titlecase` | `title`, `tc`          | Convert text to smart Title Case                 |
| `slug`      |                        | Slugify text into URL/filename-friendly form     |
| `camel`     | `camelcase`            | Convert text to `camelCase`                       |
| `pascal`    | `pascalcase`, `upper-camel` | Convert text to `PascalCase`                |
| `snake`     | `snakecase`            | Convert text to `snake_case`                      |
| `kebab`     | `kebabcase`            | Convert text to `kebab-case`                      |
| `constant`  | `scream`, `screaming-snake`, `const` | Convert text to `CONSTANT_CASE`   |
| `mock`      | `spongebob`, `mocking`, `sarcasm`, `alternate` | `mOcKiNg` aLtErNaTiNg case |
| `upper`     | `uc`                   | Convert text to `UPPERCASE`                        |
| `lower`     | `lc`                   | Convert text to `lowercase`                        |

Run `tt --help` (or `tt <op> --help`) for the full, colorized list
of options.

#### `squeeze`

Collapses excess whitespace. More aggressive than `clean`: by default *every*
run of whitespace (spaces, tabs, `\r`, `\n`, NBSP, …) becomes a single space and
the text is flattened onto one line.

| Flag                    | Effect                                              |
|-------------------------|-----------------------------------------------------|
| `--keep-newlines`       | Keep line breaks; only collapse horizontal space    |
| `--max-newlines [N]`    | Keep at most N consecutive newlines (default 2)     |
| `--remove-all`          | Delete all whitespace instead of collapsing         |
| `--no-trailing-newline` | Do not append a trailing newline                    |

```sh
printf '  a\t\tb\r\n\n  c  '  | tt squeeze                  # a b c
printf 'a b\tc\nd'            | tt squeeze --remove-all      # abcd
printf 'a\n\n\n\n\nb'         | tt squeeze --max-newlines    # a<blank>b (2 newlines)
printf 'a\n\n\n\nb'           | tt squeeze --max-newlines 3  # a + 3 newlines + b
```

#### `titlecase`

Smart title casing: minor words (`a`, `an`, `the`, `of`, `to`, …) stay lowercase
unless they are the first/last word or begin a subtitle after a colon; known
acronyms are capitalized (`nasa` → `NASA`); already-capitalized words (`iPhone`,
or a word you capitalized on purpose like `In`) are respected; hyphenated
compounds are capitalized part-by-part; leading/trailing whitespace is stripped
while interior spacing and line breaks are kept.

| Flag                | Effect                                                    |
|---------------------|-----------------------------------------------------------|
| `--no-respect-caps` | Re-case already-capitalized words instead of keeping them |
| `--acronyms LIST`   | Extra comma-separated acronyms to capitalize              |
| `--no-acronyms`     | Disable acronym capitalization                            |

```sh
echo '  the nasa api: a tale of two-cities  ' | tt titlecase
# The NASA API: A Tale of Two-Cities

echo 'the iPhone ERA' | tt titlecase                    # The iPhone ERA
echo 'the iPhone ERA' | tt titlecase --no-respect-caps  # The Iphone Era
echo 'the tui repl'   | tt titlecase --acronyms tui,repl # The TUI REPL
```

#### `slug`

Lowercases each line, replaces non-alphanumeric runs with a single separator,
and trims leading/trailing separators.

| Flag              | Effect                                              |
|-------------------|-----------------------------------------------------|
| `-s`, `--sep SEP` | Separator to join words with (default `-`)          |
| `--unicode`       | Keep all Unicode alphanumerics instead of ASCII only|

```sh
echo 'Hello, World!' | tt slug              # hello-world
echo 'My Post Title' | tt slug --sep _      # my_post_title
```

#### `clean`

Normalizes line endings to LF, applies Unicode NFC, strips control/zero-width
characters, removes trailing whitespace, squeezes repeated spaces (preserving
indentation), fixes casing (recognized acronyms → uppercase, standalone single
letters → uppercase, first letter of each sentence → uppercase, while respecting
already-capitalized words), collapses runs of blank lines, and ends with a single
newline.

| Flag                        | Effect                                         |
|-----------------------------|------------------------------------------------|
| `--ascii`                   | Fold smart quotes/dashes/ellipses to ASCII     |
| `--no-squeeze`              | Keep repeated spaces                           |
| `--no-capitalize-singles`   | Do not capitalize standalone single letters    |
| `--no-capitalize-sentences` | Do not capitalize the first letter of sentences|
| `--no-trailing-punctuation` | Strip trailing `. , ; : ! ?` from each line    |
| `--acronyms LIST`           | Extra comma-separated acronyms to capitalize   |
| `--no-acronyms`             | Disable acronym capitalization                 |
| `--no-respect-caps`         | Fold shouting ALL-CAPS words back to lowercase |
| `--keep-blank-lines`        | Keep consecutive blank lines                   |
| `--no-trailing-newline`     | Do not force a trailing newline                |

Each flag's `--help` entry includes an example transformation.

```sh
echo 'i visit nasa. i learn'  | tt clean                          # I visit NASA. I learn
echo 'Hello world.'           | tt clean --no-trailing-punctuation # Hello world
echo 'THIS is LOUD'           | tt clean --no-respect-caps         # This is loud
```

#### Identifier cases (`camel`, `pascal`, `snake`, `kebab`, `constant`)

These share a word splitter that understands spaces, `_`/`-`/`.` delimiters, and
`camelCase`/`ACRONYM` boundaries, then re-join in the target style.

```sh
echo 'get HTTP response' | tt camel      # getHttpResponse
echo 'Max Retry Count'   | tt constant   # MAX_RETRY_COUNT
echo 'helloWorldFooBar'  | tt kebab      # hello-world-foo-bar
```

#### `mock`

aLtErNaTiNg case, jUsT fOr FuN. Use `--start-upper` to begin uppercase.

```sh
echo 'this is fine' | tt mock            # tHiS iS fInE
```

### Examples

```sh
# Uppercase a file
tt upper notes.txt

# Lowercase from a pipe
echo 'HELLO' | tt lower

# Write the result to a file
tt upper input.txt -o SHOUTING.txt
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

## Versioning

Follows [semantic versioning](https://semver.org): `MAJOR.MINOR.PATCH`.

- **PATCH** — backward-compatible bug fixes. The `pre-commit` hook bumps this
  automatically on every commit, so ordinary commits need no manual step.
- **MINOR** — backward-compatible new features (a new subcommand or flag).
- **MAJOR** — breaking changes (renamed/removed command or flag, changed
  default output). Pre-1.0, a **minor** bump (`0.x`) is used for breaking
  changes, per semver's `0.y` convention.

Because the hook always bumps the patch, cutting an exact minor/major version
uses a helper that sets the version and commits with `--no-verify` (bypassing
the hook), then tags it:

```sh
make release VERSION=0.2.0      # commit "Release v0.2.0" + tag v0.2.0
git push origin main --tags     # publish the release
```

## License

[MIT](LICENSE) © Zach Banks
