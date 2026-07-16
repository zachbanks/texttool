//! `clean` — tidy up messy text (the former `textclean` script).
//!
//! The default pipeline is a conservative "make this text sane" pass:
//!
//! 1. Normalize line endings (CRLF / lone CR) to `\n`.
//! 2. Apply Unicode NFC normalization so equivalent forms compare equal.
//! 3. Strip control characters (except tab/newline) and zero-width characters.
//! 4. Remove trailing whitespace from every line and squeeze runs of spaces.
//! 5. Collapse three-or-more consecutive newlines down to a single blank line.
//! 6. Trim leading/trailing blank lines and end with exactly one newline.
//!
//! Every potentially-destructive step has a flag to turn it off, and `--ascii`
//! additionally folds common "smart" punctuation to plain ASCII.

use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};
use unicode_normalization::UnicodeNormalization;

/// Whitespace/control cleanup for text.
pub struct Clean;

/// Zero-width and joiner characters that are invisible but change byte length.
const ZERO_WIDTH: &[char] = &[
    '\u{200B}', // zero-width space
    '\u{200C}', // zero-width non-joiner
    '\u{200D}', // zero-width joiner
    '\u{2060}', // word joiner
    '\u{FEFF}', // zero-width no-break space / BOM
];

impl Clean {
    /// Normalize CRLF and lone CR line endings to `\n`.
    fn normalize_newlines(input: &str) -> String {
        input.replace("\r\n", "\n").replace('\r', "\n")
    }

    /// Fold common "smart" punctuation to ASCII equivalents.
    fn to_ascii_punct(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{2033}' => '"',
                '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '\u{2032}' => '\'',
                '\u{2013}' | '\u{2014}' | '\u{2012}' | '\u{2015}' => '-',
                '\u{00A0}' | '\u{2007}' | '\u{202F}' => ' ',
                other => other,
            })
            .collect::<String>()
            // The ellipsis is one char mapping to three, handled separately.
            .replace('\u{2026}', "...")
    }

    /// Drop control characters (keeping tab and newline) and zero-width chars.
    fn strip_invisibles(input: &str) -> String {
        input
            .chars()
            .filter(|&c| {
                if ZERO_WIDTH.contains(&c) {
                    return false;
                }
                !c.is_control() || c == '\n' || c == '\t'
            })
            .collect()
    }

    /// Strip trailing whitespace and optionally squeeze internal space runs.
    ///
    /// Leading indentation (spaces and tabs) is preserved so that list markers,
    /// code, and other intentionally-indented lines survive; squeezing only
    /// applies to interior space runs.
    fn tidy_line(line: &str, squeeze: bool) -> String {
        let trimmed = line.trim_end();
        if !squeeze {
            return trimmed.to_string();
        }
        // Keep the leading indentation verbatim, squeeze the remainder.
        let body_start = trimmed.len() - trimmed.trim_start_matches([' ', '\t']).len();
        let (indent, body) = trimmed.split_at(body_start);

        let mut out = String::with_capacity(trimmed.len());
        out.push_str(indent);
        let mut prev_space = false;
        for c in body.chars() {
            if c == ' ' {
                if !prev_space {
                    out.push(c);
                }
                prev_space = true;
            } else {
                out.push(c);
                prev_space = false;
            }
        }
        out
    }

    /// Collapse runs of blank lines to at most one blank line.
    fn collapse_blank_lines(lines: Vec<String>) -> Vec<String> {
        let mut out: Vec<String> = Vec::with_capacity(lines.len());
        let mut blank_run = false;
        for line in lines {
            if line.is_empty() {
                if blank_run {
                    continue;
                }
                blank_run = true;
            } else {
                blank_run = false;
            }
            out.push(line);
        }
        out
    }
}

impl Transform for Clean {
    fn name(&self) -> &'static str {
        "clean"
    }

    fn about(&self) -> &'static str {
        "Tidy whitespace, line endings, and invisible characters"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Clean up messy text. By default this normalizes line endings to LF, \
             applies Unicode NFC normalization, removes control and zero-width \
             characters, strips trailing whitespace, squeezes repeated spaces, \
             collapses blocks of blank lines, and ends the output with a single \
             newline. Use the flags to disable individual steps.",
        )
    }

    fn augment(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("ascii")
                .long("ascii")
                .help("Fold smart quotes, dashes, and ellipses to plain ASCII")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-squeeze")
                .long("no-squeeze")
                .help("Keep repeated spaces instead of collapsing them")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("keep-blank-lines")
                .long("keep-blank-lines")
                .help("Keep consecutive blank lines instead of collapsing them")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-trailing-newline")
                .long("no-trailing-newline")
                .help("Do not force a single trailing newline")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let squeeze = !args.get_flag("no-squeeze");
        let collapse = !args.get_flag("keep-blank-lines");
        let trailing_newline = !args.get_flag("no-trailing-newline");

        let mut text = Self::normalize_newlines(input);
        if args.get_flag("ascii") {
            text = Self::to_ascii_punct(&text);
        }
        text = text.nfc().collect::<String>();
        text = Self::strip_invisibles(&text);

        let mut lines: Vec<String> = text
            .split('\n')
            .map(|l| Self::tidy_line(l, squeeze))
            .collect();
        if collapse {
            lines = Self::collapse_blank_lines(lines);
        }

        // Trim leading and trailing blank lines.
        while lines.first().is_some_and(|l| l.is_empty()) {
            lines.remove(0);
        }
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }

        let mut result = lines.join("\n");
        if trailing_newline && !result.is_empty() {
            result.push('\n');
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    /// Parse `clean`'s flags from an argv-style slice for testing.
    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Clean.augment(Command::new("clean"));
        let mut argv = vec!["clean"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    #[test]
    fn normalizes_crlf() {
        assert_eq!(Clean.apply("a\r\nb\r\n", &args(&[])).unwrap(), "a\nb\n");
    }

    #[test]
    fn strips_trailing_whitespace_and_squeezes_spaces() {
        assert_eq!(
            Clean.apply("hello    world   \n", &args(&[])).unwrap(),
            "hello world\n"
        );
    }

    #[test]
    fn no_squeeze_keeps_runs() {
        assert_eq!(
            Clean.apply("a    b", &args(&["--no-squeeze"])).unwrap(),
            "a    b\n"
        );
    }

    #[test]
    fn collapses_blank_line_runs() {
        assert_eq!(Clean.apply("a\n\n\n\nb\n", &args(&[])).unwrap(), "a\n\nb\n");
    }

    #[test]
    fn keep_blank_lines_flag() {
        assert_eq!(
            Clean
                .apply("a\n\n\n\nb", &args(&["--keep-blank-lines"]))
                .unwrap(),
            "a\n\n\n\nb\n"
        );
    }

    #[test]
    fn trims_surrounding_blank_lines_but_keeps_indentation() {
        // Leading/trailing blank lines are dropped; leading indentation on a
        // content line is preserved, trailing whitespace stripped.
        assert_eq!(Clean.apply("\n\n  hi  \n\n", &args(&[])).unwrap(), "  hi\n");
    }

    #[test]
    fn squeeze_preserves_leading_indentation() {
        assert_eq!(Clean.apply("    a    b", &args(&[])).unwrap(), "    a b\n");
    }

    #[test]
    fn removes_zero_width_and_control_chars() {
        let input = "a\u{200B}b\u{0007}c";
        assert_eq!(Clean.apply(input, &args(&[])).unwrap(), "abc\n");
    }

    #[test]
    fn ascii_flag_folds_smart_punctuation() {
        let input = "\u{201C}quote\u{201D} \u{2014} it\u{2019}s fine\u{2026}";
        assert_eq!(
            Clean.apply(input, &args(&["--ascii"])).unwrap(),
            "\"quote\" - it's fine...\n"
        );
    }

    #[test]
    fn ascii_flag_replaces_nbsp_with_space() {
        assert_eq!(
            Clean.apply("a\u{00A0}b", &args(&["--ascii"])).unwrap(),
            "a b\n"
        );
    }

    #[test]
    fn empty_input_stays_empty() {
        assert_eq!(Clean.apply("", &args(&[])).unwrap(), "");
    }

    #[test]
    fn no_trailing_newline_flag() {
        assert_eq!(
            Clean
                .apply("hi\n", &args(&["--no-trailing-newline"]))
                .unwrap(),
            "hi"
        );
    }
}
