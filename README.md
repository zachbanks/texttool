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
| `strip`     | `text-strip`           | Strip decorative punctuation/noise from edges    |
| `extract`   | `infoparse`            | Pull phones/emails/dates/… into Markdown sections|
| `titlecase` | `title`, `tc`          | Convert text to smart Title Case                 |
| `slug`      |                        | Slugify text into URL/filename-friendly form     |
| `unslug`    | `deslug`               | Split slugs/identifiers into spaced words        |
| `humanize`  | `readable`             | Filename/slug → clean readable text              |
| `replace`   | `sub`                  | Find and replace text (literal or regex)         |
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

#### `strip`

Removes decorative/markup punctuation (quotes, markdown rules, bullets,
separators) plus whitespace and control chars from the **edges** of the text,
leaving interior content untouched. Sentence-ending `.`/`!`/`?` are kept unless
`--aggressive`.

| Flag                | Effect                                                    |
|---------------------|-----------------------------------------------------------|
| `--no-punct`        | Only trim whitespace/control chars; keep edge punctuation |
| `--aggressive`      | Also strip edge sentence punctuation (`. ! ?`)            |
| `--collapse-blanks` | Collapse runs of blank lines into one                     |
| `-s`, `--squeeze`   | Collapse interior whitespace; strip per-line indentation  |
| `-l`, `--strip-lines`| Strip decorative tokens on every line, not just the edges|
| `-n`, `--no-newline`| Do not append a trailing newline                          |

```sh
echo '  --- hello ---  ' | tt strip                  # hello
echo '"Done."'           | tt strip                  # Done.  (sentence dot kept)
echo '"Done."'           | tt strip --aggressive     # Done
printf '*** heading ***' | tt strip --strip-lines    # heading
```

#### `extract`

Pulls structured information out of text using per-category regular expressions,
grouped under Markdown headings. Built-in categories: Phone Numbers, Emails,
URLs, Addresses (rough), Dates, Times, SSNs, Credit Cards, IP Addresses.

| Flag                 | Effect                                              |
|----------------------|-----------------------------------------------------|
| `--only LIST`        | Only these categories (e.g. `--only emails,phones`) |
| `--no-headers`       | Print values only, no `# Heading` lines             |
| `--show-empty`       | Include categories that had no matches              |
| `--no-dedup`         | Keep duplicate matches                              |
| `--patterns-file P`  | Load extra/override categories from a TOML file     |
| `--list`             | List available categories and their patterns        |

```sh
echo 'Call 415-555-0198 or jim@example.com by 12/25/2026' | tt extract
# # Phone Numbers
# 415-555-0198
#
# # Emails
# jim@example.com
#
# # Dates
# 12/25/2026
```

**Configurable categories.** The recognized set is the built-ins plus a TOML
config, from `$TEXTTOOL_PATTERNS_FILE`, else
`$XDG_CONFIG_HOME/texttool/patterns.toml`, else
`~/.config/texttool/patterns.toml` (or `--patterns-file`). A `[[category]]` whose
`name` matches a built-in overrides its regex, `enabled = false` disables one, and
a new name appends a category. See [`examples/patterns.toml`](examples/patterns.toml).

If a pattern has a **capturing group**, only the first group is reported — so a
pattern can anchor on a label yet output just the value:

```toml
[[category]]
name = "Order Numbers"
regex = '''(?i)\border\s*(?:#|no\.?|number)\s*:?\s*([A-Za-z0-9-]*\d[A-Za-z0-9-]*)'''
# "Order # WK32338412" -> WK32338412
```

> Patterns use Rust's `regex` crate syntax — **no lookaround or backreferences**.
> Address matching is a best-effort heuristic; the extractor finds patterns but
> does not validate them (no Luhn check on cards).

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
| `--acronyms-file P` | Read extra acronyms from a file                           |
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

#### `unslug`

The inverse of `slug`: splits on separators (`-`, `_`, `.`, …) and
`camelCase`/`ACRONYM` boundaries and joins the words with spaces, preserving
case.

```sh
echo 'Thorne-magnesium-2026-07-17' | tt unslug   # Thorne magnesium 2026 07 17
echo 'getHTTPResponse'             | tt unslug   # get HTTP Response
```

#### `humanize`

Turns a filename or slug into clean, readable text: drops a trailing file
extension, splits on every common filename delimiter (`-`, `_`, `.`, spaces) and
`camelCase`/`ACRONYM` boundaries, then cleans whitespace and casing. Composes
`unslug` + `clean`.

```sh
echo 'Thorne-magnesium-receipt-2026-07-17.pdf' | tt humanize
# Thorne magnesium receipt 2026 07 17
echo 'annual_api_report.docx' | tt humanize        # Annual API report
echo 'getHTTPResponse.log'     | tt humanize        # Get HTTP Response
```

#### `replace`

General find-and-replace. `FROM` is literal by default, or a regex with
`--regex` (then `TO` may use `$1` group references). Pass an empty `TO` (`''`) to
delete. Reads file operands or stdin.

| Flag                | Effect                                    |
|---------------------|-------------------------------------------|
| `-r`, `--regex`     | Treat `FROM` as a regular expression      |
| `-i`, `--ignore-case`| Match case-insensitively                 |

```sh
echo 'a-b-c'      | tt replace - ' '                       # a b c
echo 'a-b_c.d'    | tt replace -r '[-_.]+' ' '             # a b c d
echo '2026-07-17' | tt replace -r '(\d+)-(\d+)-(\d+)' '$3/$2/$1'  # 17/07/2026
```

#### `clean`

Normalizes line endings to LF, applies Unicode NFC, strips control/zero-width
characters, removes trailing whitespace, squeezes repeated spaces (preserving
indentation), fixes casing (recognized acronyms → uppercase, standalone single
letters → uppercase, first letter of each sentence → uppercase — sentence starts
only, so wrapped lines aren't over-capitalized — while respecting
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
| `--acronyms-file P`         | Read extra acronyms from a file                |
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

#### Configuring acronyms

Both `clean` and `titlecase` capitalize recognized acronyms. The recognized set
is the built-in list plus anything you configure, from three layered sources:

1. **A persistent config file** — so you don't retype `--acronyms` every time.
   The first of these that exists is loaded:
   - `$TEXTTOOL_ACRONYMS_FILE`
   - `$XDG_CONFIG_HOME/texttool/acronyms.txt`
   - `~/.config/texttool/acronyms.txt`

   One acronym per line (or comma/space separated); `#` starts a comment:
   ```
   # ~/.config/texttool/acronyms.txt
   tui, repl
   yolo   # add whatever you like
   ```
2. **A per-invocation file**: `--acronyms-file <PATH>`.
3. **Inline**: `--acronyms a,b,c`.

`--no-acronyms` disables acronym capitalization entirely (built-ins included).

```sh
echo 'the tui repl' | tt titlecase --acronyms tui,repl   # The TUI REPL
echo 'the api docs' | tt titlecase --no-acronyms          # The Api Docs
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
the hook), tags it, pushes, and creates a GitHub Release with generated notes:

```sh
make release VERSION=0.2.0      # commit + tag + push + `gh release create`
```

Releases are published at
[github.com/zachbanks/texttool/releases](https://github.com/zachbanks/texttool/releases).

## License

[MIT](LICENSE) © Zach Banks
