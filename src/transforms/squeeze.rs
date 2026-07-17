//! `squeeze` — collapse excess whitespace.
//!
//! More aggressive than `clean`: by default every run of whitespace of any kind
//! (spaces, tabs, `\r`, `\n`, and Unicode spaces such as NBSP) is collapsed to a
//! single space and the ends are trimmed, flattening the text onto one line.
//!
//! - `--keep-newlines` collapses only horizontal whitespace, preserving line
//!   breaks between non-empty lines.
//! - `--remove-all` deletes whitespace entirely, joining everything together.

use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Whitespace-collapsing transform.
pub struct Squeeze;

/// Collapse every run of whitespace into a single space and trim the ends.
fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            pending_space = true;
        } else {
            // Only emit a separator between two non-whitespace chunks, which
            // trims leading (out empty) and trailing (loop ends) whitespace.
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        }
    }
    out
}

/// Squeeze horizontal whitespace per line and cap runs of consecutive newlines
/// at `max` (so at most `max - 1` blank lines survive between content), trimming
/// leading and trailing blank lines.
fn cap_newlines(input: &str, max: usize) -> String {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let allowed_blanks = max.saturating_sub(1);

    let mut lines: Vec<String> = Vec::new();
    let mut blank_run = 0usize;
    for raw in normalized.split('\n') {
        let line = collapse_ws(raw);
        if line.is_empty() {
            blank_run += 1;
            if blank_run <= allowed_blanks {
                lines.push(String::new());
            }
        } else {
            blank_run = 0;
            lines.push(line);
        }
    }

    while lines.first().is_some_and(|l| l.is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

impl Transform for Squeeze {
    fn name(&self) -> &'static str {
        "squeeze"
    }

    fn about(&self) -> &'static str {
        "Collapse excess spaces, tabs, and newlines"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Collapse excess whitespace. By default every run of whitespace \
             (spaces, tabs, carriage returns, newlines, and Unicode spaces) is \
             replaced by a single space and the ends are trimmed, flattening the \
             input onto one line. Use --keep-newlines to keep line breaks \
             between non-empty lines, or --remove-all to strip whitespace \
             entirely.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["sq", "normalize-ws"]
    }

    fn augment(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("keep-newlines")
                .long("keep-newlines")
                .help(
                    "Keep line breaks, collapse horizontal space [e.g. \"a\\n\\nb\" -> \"a\\nb\"]",
                )
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("max-newlines")
                .long("max-newlines")
                .value_name("N")
                .help("Keep at most N consecutive newlines [e.g. --max-newlines 2: 5 -> 2]")
                .num_args(0..=1)
                .default_missing_value("2")
                .value_parser(clap::value_parser!(u64).range(1..))
                .conflicts_with("keep-newlines"),
        )
        .arg(
            Arg::new("remove-all")
                .long("remove-all")
                .help("Remove all whitespace [e.g. \"a b\\tc\" -> \"abc\"]")
                .conflicts_with_all(["keep-newlines", "max-newlines"])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-trailing-newline")
                .long("no-trailing-newline")
                .help("Do not append a trailing newline [e.g. \"a b\\n\" -> \"a b\"]")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let remove_all = args.get_flag("remove-all");
        let keep_newlines = args.get_flag("keep-newlines");
        let max_newlines = args.get_one::<u64>("max-newlines").copied();
        let trailing_newline = !args.get_flag("no-trailing-newline");

        let mut body = if remove_all {
            input.chars().filter(|c| !c.is_whitespace()).collect()
        } else if let Some(max) = max_newlines {
            cap_newlines(input, max as usize)
        } else if keep_newlines {
            // Normalize CR/CRLF first, then squeeze each line and drop blanks.
            input
                .replace("\r\n", "\n")
                .replace('\r', "\n")
                .split('\n')
                .map(collapse_ws)
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            collapse_ws(input)
        };

        if trailing_newline && !body.is_empty() {
            body.push('\n');
        }
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Squeeze.augment(Command::new("squeeze"));
        let mut argv = vec!["squeeze"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    #[test]
    fn flattens_all_whitespace_to_one_line() {
        assert_eq!(
            Squeeze.apply("  a\t\tb\r\n\n  c  ", &args(&[])).unwrap(),
            "a b c\n"
        );
    }

    #[test]
    fn collapses_unicode_spaces() {
        assert_eq!(
            Squeeze.apply("a\u{00A0}\u{00A0}b", &args(&[])).unwrap(),
            "a b\n"
        );
    }

    #[test]
    fn keep_newlines_preserves_line_breaks() {
        assert_eq!(
            Squeeze
                .apply("  a   b \n\n\n c \t d \n", &args(&["--keep-newlines"]))
                .unwrap(),
            "a b\nc d\n"
        );
    }

    #[test]
    fn remove_all_strips_every_whitespace() {
        assert_eq!(
            Squeeze
                .apply("a b\tc\nd", &args(&["--remove-all"]))
                .unwrap(),
            "abcd\n"
        );
    }

    #[test]
    fn max_newlines_defaults_to_two() {
        // Flag given without a value keeps at most 2 consecutive newlines.
        assert_eq!(
            Squeeze
                .apply("a\n\n\n\n\nb", &args(&["--max-newlines"]))
                .unwrap(),
            "a\n\nb\n"
        );
    }

    #[test]
    fn max_newlines_explicit_value() {
        assert_eq!(
            Squeeze
                .apply("a\n\n\n\n\nb", &args(&["--max-newlines", "3"]))
                .unwrap(),
            "a\n\n\nb\n"
        );
    }

    #[test]
    fn max_newlines_one_collapses_blanks() {
        assert_eq!(
            Squeeze
                .apply("a\n\n\nb", &args(&["--max-newlines", "1"]))
                .unwrap(),
            "a\nb\n"
        );
    }

    #[test]
    fn max_newlines_squeezes_horizontal_and_trims_edges() {
        assert_eq!(
            Squeeze
                .apply("\n\n  a   b  \n\n\n\n c \n\n", &args(&["--max-newlines"]))
                .unwrap(),
            "a b\n\nc\n"
        );
    }

    #[test]
    fn no_trailing_newline_flag() {
        assert_eq!(
            Squeeze
                .apply("a  b", &args(&["--no-trailing-newline"]))
                .unwrap(),
            "a b"
        );
    }

    #[test]
    fn empty_input_stays_empty() {
        assert_eq!(Squeeze.apply("", &args(&[])).unwrap(), "");
        assert_eq!(Squeeze.apply("   \n\t ", &args(&[])).unwrap(), "");
    }
}
