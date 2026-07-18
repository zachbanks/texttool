//! `strip` — remove decorative noise from the edges of text (ports the
//! standalone `text_strip` script).
//!
//! Unlike `clean` (which tidies whitespace and casing) and `squeeze` (which
//! collapses whitespace), `strip` removes *decorative* punctuation — markdown
//! rules, bullets, wrapping quotes, separators — from the very start and end of
//! the text, leaving the real content untouched in the middle. Sentence-ending
//! `.`/`!`/`?` are kept unless `--aggressive` is given.

use crate::casing::segment;
use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashSet;

/// Decorative / markup punctuation rarely meaningful at the edge of content.
const DECORATIVE: &[char] = &[
    '"', '\'', '`', '*', '_', '~', '#', '>', '=', '+', '|', '\\', '/', '-', '\u{2013}', '\u{2014}',
    '\u{2022}', '\u{25E6}', '\u{00B7}', '\u{25CF}', '\u{25CB}', '\u{25AA}', '\u{25AB}', '\u{25A0}',
    '\u{25A1}', '\u{00BB}', '\u{00AB}', '\u{2039}', '\u{203A}', ',', ';', ':', '\u{201C}',
    '\u{201D}', '\u{2018}', '\u{2019}', '\u{2026}',
];

/// Sentence terminators, only stripped from the edges in `--aggressive` mode.
const SENTENCE_END: &[char] = &['.', '!', '?'];

/// Edge-noise stripping for text.
pub struct Strip;

/// Options controlling a strip pass, mirroring the CLI flags.
struct Opts {
    strip_punct: bool,
    aggressive: bool,
    collapse_blanks: bool,
    squeeze: bool,
    strip_lines: bool,
}

/// Normalize CRLF and lone CR to `\n`.
fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Drop the control characters `text_strip` removes (keeping tab and newline).
fn is_dropped_control(c: char) -> bool {
    let n = c as u32;
    n <= 0x08 || n == 0x0b || n == 0x0c || (0x0e..=0x1f).contains(&n) || n == 0x7f
}

/// Collapse interior whitespace runs to a single space and trim the ends.
fn squeeze_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut pending_space = false;
    for c in line.chars() {
        if c.is_whitespace() {
            pending_space = true;
        } else {
            if pending_space && !out.is_empty() {
                out.push(' ');
            }
            pending_space = false;
            out.push(c);
        }
    }
    out
}

/// Collapse runs of 2+ ASCII spaces into one (leaving other whitespace alone).
fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

/// Strip whitespace and the given edge chars from both ends, alternating until
/// stable so interleaved noise like `"  -- "` is fully removed.
fn strip_edges(text: &str, chars: &HashSet<char>) -> String {
    if chars.is_empty() {
        return text.trim().to_string();
    }
    let mut cur = text.to_string();
    loop {
        let next = cur
            .trim()
            .trim_start_matches(|c| chars.contains(&c))
            .trim_end_matches(|c| chars.contains(&c))
            .to_string();
        if next == cur {
            return next;
        }
        cur = next;
    }
}

/// Remove whitespace-delimited tokens made entirely of decorative chars,
/// preserving the surrounding whitespace.
fn remove_decorative_tokens(line: &str, chars: &HashSet<char>) -> String {
    segment(line)
        .into_iter()
        .map(|seg| {
            if seg.is_word && seg.text.chars().all(|c| chars.contains(&c)) {
                String::new()
            } else {
                seg.text
            }
        })
        .collect()
}

/// The full strip pipeline, ported from `text_strip.clean_text`.
fn strip_text(input: &str, opts: &Opts) -> String {
    let text: String = normalize_newlines(input)
        .chars()
        .filter(|&c| !is_dropped_control(c))
        .collect();

    let mut lines: Vec<String> = text
        .split('\n')
        .map(|line| {
            if opts.squeeze {
                squeeze_line(line)
            } else {
                line.trim_end().to_string()
            }
        })
        .collect();

    let mut edge_chars: HashSet<char> = DECORATIVE.iter().copied().collect();
    if opts.aggressive {
        edge_chars.extend(SENTENCE_END.iter().copied());
    }

    if opts.strip_punct && opts.strip_lines {
        lines = lines
            .into_iter()
            .map(|line| {
                let line = remove_decorative_tokens(&line, &edge_chars);
                let line = strip_edges(&line, &edge_chars);
                collapse_spaces(&line)
            })
            .collect();
    }

    let mut text = lines.join("\n");

    if opts.collapse_blanks {
        // Collapse runs of 3+ newlines down to exactly two (one blank line).
        while text.contains("\n\n\n") {
            text = text.replace("\n\n\n", "\n\n");
        }
    }

    if opts.strip_punct {
        strip_edges(&text, &edge_chars)
    } else {
        text.trim().to_string()
    }
}

impl Transform for Strip {
    fn name(&self) -> &'static str {
        "strip"
    }

    fn about(&self) -> &'static str {
        "Strip decorative punctuation and noise from the edges of text [e.g. \"-- hello --\" -> \"hello\"]"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Remove decorative/markup punctuation (quotes, rules, bullets, \
             separators), whitespace, carriage returns, and control characters \
             from the start and end of text, leaving interior content untouched. \
             Sentence-ending . ! ? are kept unless --aggressive is given.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["text-strip"]
    }

    fn augment(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("no-punct")
                .long("no-punct")
                .help("Strip only whitespace/control chars [e.g. \"-- hi --\" -> \"-- hi --\" trimmed]")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("aggressive")
                .long("aggressive")
                .help("Also strip edge sentence punctuation [e.g. \"Done.\" -> \"Done\"]")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("collapse-blanks")
                .long("collapse-blanks")
                .help("Collapse blank-line runs [e.g. \"a\\n\\n\\n\\nb\" -> \"a\\n\\nb\"]")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("squeeze")
                .short('s')
                .long("squeeze")
                .help("Collapse interior whitespace, strip line indentation [e.g. \"a   b\" -> \"a b\"]")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("strip-lines")
                .short('l')
                .long("strip-lines")
                .help("Strip decorative tokens on every line [e.g. \"*** hi ***\" -> \"hi\"]")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-newline")
                .short('n')
                .long("no-newline")
                .help("Do not append a trailing newline [e.g. \"hi\" not \"hi\\n\"]")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let opts = Opts {
            strip_punct: !args.get_flag("no-punct"),
            aggressive: args.get_flag("aggressive"),
            collapse_blanks: args.get_flag("collapse-blanks"),
            squeeze: args.get_flag("squeeze"),
            strip_lines: args.get_flag("strip-lines"),
        };
        let mut result = strip_text(input, &opts);
        if !args.get_flag("no-newline") && !result.is_empty() {
            result.push('\n');
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Strip.augment(Command::new("strip"));
        let mut argv = vec!["strip"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    fn strip(input: &str, extra: &[&str]) -> String {
        Strip.apply(input, &args(extra)).unwrap()
    }

    #[test]
    fn strips_decorative_edges() {
        assert_eq!(strip("  --- hello ---  ", &[]), "hello\n");
        assert_eq!(strip("  *** title ***  ", &[]), "title\n");
        assert_eq!(strip("\"  quoted  \"", &[]), "quoted\n");
    }

    #[test]
    fn keeps_sentence_punctuation_by_default() {
        assert_eq!(strip("Done.", &[]), "Done.\n");
        assert_eq!(strip("Really?!", &[]), "Really?!\n");
    }

    #[test]
    fn aggressive_strips_sentence_punctuation() {
        assert_eq!(strip("Done.", &["--aggressive"]), "Done\n");
    }

    #[test]
    fn preserves_interior_content() {
        assert_eq!(strip("a - b - c", &[]), "a - b - c\n");
        assert_eq!(strip("well-known", &[]), "well-known\n");
    }

    #[test]
    fn no_punct_only_trims_whitespace() {
        assert_eq!(strip("  -- hi --  ", &["--no-punct"]), "-- hi --\n");
    }

    #[test]
    fn squeeze_collapses_interior_whitespace() {
        assert_eq!(strip("  a    b   c  ", &["--squeeze"]), "a b c\n");
    }

    #[test]
    fn strip_lines_removes_decorative_tokens_anywhere() {
        assert_eq!(strip("*** heading ***", &["--strip-lines"]), "heading\n");
        assert_eq!(strip("*** a ***\n--- b ---", &["--strip-lines"]), "a\nb\n");
        // Tokens with letters attached are kept.
        assert_eq!(
            strip("keep well-known here", &["--strip-lines"]),
            "keep well-known here\n"
        );
    }

    #[test]
    fn collapse_blanks_flag() {
        assert_eq!(strip("a\n\n\n\nb", &["--collapse-blanks"]), "a\n\nb\n");
    }

    #[test]
    fn no_newline_flag() {
        assert_eq!(strip("  hi  ", &["--no-newline"]), "hi");
    }

    #[test]
    fn normalizes_crlf_and_drops_control_chars() {
        assert_eq!(strip("a\r\nb\u{0007}", &[]), "a\nb\n");
    }

    #[test]
    fn empty_after_stripping_stays_empty() {
        assert_eq!(strip("  ***  ", &[]), "");
    }
}
