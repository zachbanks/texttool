//! `clean` — tidy up messy text (the former `textclean` script).
//!
//! The default pipeline is a conservative "make this text sane" pass:
//!
//! 1. Normalize line endings (CRLF / lone CR) to `\n`.
//! 2. Apply Unicode NFC normalization so equivalent forms compare equal.
//! 3. Strip control characters (except tab/newline) and zero-width characters.
//! 4. Remove trailing whitespace from every line and squeeze runs of spaces.
//! 5. Fix casing (respecting already-capitalized words): capitalize recognized
//!    acronyms (`api` -> `API`), standalone single letters (`i` -> `I`), and the
//!    first letter of each sentence.
//! 6. Collapse three-or-more consecutive newlines down to a single blank line.
//! 7. Trim leading/trailing blank lines and end with exactly one newline.
//!
//! Every step has a flag to turn it off; `--ascii` folds "smart" punctuation to
//! plain ASCII, `--no-trailing-punctuation` strips sentence punctuation from
//! line ends, `--acronyms`/`--no-acronyms` tune acronym handling, and
//! `--no-respect-caps` lets clean fold shouting ALL-CAPS words back to
//! lowercase.

use crate::acronyms::{add_acronym_args, build_acronym_set};
use crate::casing::{
    AcronymSet, capitalize_first_alpha, capitalize_sentences, has_uppercase, is_all_caps,
    is_single_letter, segment,
};
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

/// Sentence punctuation stripped from line ends by `--no-trailing-punctuation`.
const TRAILING_PUNCT: &[char] = &['.', ',', ';', ':', '!', '?', '\u{2026}'];

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

    /// Apply per-word case fixes: respect already-capitalized words, fold
    /// shouting ALL-CAPS when not respecting caps, uppercase recognized
    /// acronyms, and capitalize standalone single letters. Spacing is preserved.
    fn apply_word_casing(
        line: &str,
        respect_caps: bool,
        capitalize_singles: bool,
        acronyms: &AcronymSet,
    ) -> String {
        segment(line)
            .into_iter()
            .map(|seg| {
                if !seg.is_word {
                    return seg.text;
                }
                let mut word = seg.text;
                // Leave words the writer already capitalized alone.
                if respect_caps && has_uppercase(&word) {
                    return word;
                }
                if !respect_caps && is_all_caps(&word) {
                    word = word.to_lowercase();
                }
                if acronyms.matches(&word) {
                    word = word.to_uppercase();
                } else if capitalize_singles && is_single_letter(&word) {
                    word = capitalize_first_alpha(&word);
                }
                word
            })
            .collect()
    }

    /// Strip trailing sentence punctuation (and any whitespace before it).
    fn strip_trailing_punct(line: &str) -> String {
        line.trim_end_matches(|c: char| TRAILING_PUNCT.contains(&c) || c.is_whitespace())
            .to_string()
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
             capitalizes standalone single letters, collapses blocks of blank \
             lines, and ends the output with a single newline. Already-capitalized \
             words are respected. Use the flags to disable individual steps.",
        )
    }

    fn augment(&self, cmd: Command) -> Command {
        let cmd = cmd
            .arg(
                Arg::new("ascii")
                    .long("ascii")
                    .help("Fold smart punctuation to ASCII [e.g. \"“hi” — it’s\" -> '\"hi\" - it's']")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-squeeze")
                    .long("no-squeeze")
                    .help("Keep repeated spaces [e.g. \"a    b\" -> \"a    b\"]")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-capitalize-singles")
                    .long("no-capitalize-singles")
                    .help("Do not capitalize standalone single letters [e.g. \"say i\" -> \"say i\"]")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-capitalize-sentences")
                    .long("no-capitalize-sentences")
                    .help("Do not capitalize the first letter of sentences [e.g. \"hi. bye\" -> \"hi. bye\"]")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-trailing-punctuation")
                    .long("no-trailing-punctuation")
                    .help("Strip trailing sentence punctuation [e.g. \"Hi there.\" -> \"Hi there\"]")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-respect-caps")
                    .long("no-respect-caps")
                    .help("Fold shouting ALL-CAPS words to lowercase [e.g. \"LOUD\" -> \"loud\"]")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("keep-blank-lines")
                    .long("keep-blank-lines")
                    .help("Keep consecutive blank lines [e.g. \"a\\n\\n\\nb\" keeps both blanks]")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-trailing-newline")
                    .long("no-trailing-newline")
                    .help("Do not force a trailing newline [e.g. \"hi\\n\" -> \"hi\"]")
                    .action(ArgAction::SetTrue),
            );
        add_acronym_args(cmd)
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let squeeze = !args.get_flag("no-squeeze");
        let collapse = !args.get_flag("keep-blank-lines");
        let trailing_newline = !args.get_flag("no-trailing-newline");
        let capitalize_singles = !args.get_flag("no-capitalize-singles");
        let cap_sentences = !args.get_flag("no-capitalize-sentences");
        let respect_caps = !args.get_flag("no-respect-caps");
        let strip_punct = args.get_flag("no-trailing-punctuation");
        let acronyms = build_acronym_set(args);

        let mut text = Self::normalize_newlines(input);
        if args.get_flag("ascii") {
            text = Self::to_ascii_punct(&text);
        }
        text = text.nfc().collect::<String>();
        text = Self::strip_invisibles(&text);

        let mut lines: Vec<String> = text
            .split('\n')
            .map(|line| {
                let mut out = Self::tidy_line(line, squeeze);
                out = Self::apply_word_casing(&out, respect_caps, capitalize_singles, &acronyms);
                if strip_punct {
                    out = Self::strip_trailing_punct(&out);
                }
                out
            })
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
        // Sentence casing runs over the whole text (newlines are neutral) so it
        // only capitalizes the very start and words after . ! ? — not every
        // line start, which would wrongly capitalize wrapped prose.
        if cap_sentences {
            result = capitalize_sentences(&result);
        }
        if trailing_newline && !result.is_empty() {
            result.push('\n');
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse `clean`'s flags from an argv-style slice for testing.
    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Clean.augment(Command::new("clean"));
        let mut argv = vec!["clean"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    #[test]
    fn normalizes_crlf_and_capitalizes_single_letters() {
        // Lone single letters are capitalized by default.
        assert_eq!(Clean.apply("a\r\nb\r\n", &args(&[])).unwrap(), "A\nB\n");
    }

    #[test]
    fn strips_trailing_whitespace_and_squeezes_spaces() {
        assert_eq!(
            Clean.apply("hello    world   \n", &args(&[])).unwrap(),
            "Hello world\n"
        );
    }

    #[test]
    fn no_squeeze_keeps_runs() {
        assert_eq!(
            Clean.apply("aa    bb", &args(&["--no-squeeze"])).unwrap(),
            "Aa    bb\n"
        );
    }

    #[test]
    fn capitalizes_lone_i() {
        assert_eq!(
            Clean.apply("i think i am", &args(&[])).unwrap(),
            "I think I am\n"
        );
    }

    #[test]
    fn no_capitalize_singles_flag() {
        // Sentence casing still capitalizes the line start; the lone mid-line
        // "i" stays lowercase because single-letter casing is disabled.
        assert_eq!(
            Clean
                .apply("say i now", &args(&["--no-capitalize-singles"]))
                .unwrap(),
            "Say i now\n"
        );
    }

    #[test]
    fn capitalizes_first_letter_of_sentences() {
        assert_eq!(
            Clean.apply("hello world. another one", &args(&[])).unwrap(),
            "Hello world. Another one\n"
        );
    }

    #[test]
    fn no_capitalize_sentences_flag() {
        assert_eq!(
            Clean
                .apply("hello. world", &args(&["--no-capitalize-sentences"]))
                .unwrap(),
            "hello. world\n"
        );
    }

    #[test]
    fn capitalizes_known_acronyms() {
        assert_eq!(
            Clean.apply("visit nasa and fbi", &args(&[])).unwrap(),
            "Visit NASA and FBI\n"
        );
    }

    #[test]
    fn no_acronyms_flag_disables_them() {
        assert_eq!(
            Clean
                .apply("visit nasa", &args(&["--no-acronyms"]))
                .unwrap(),
            "Visit nasa\n"
        );
    }

    #[test]
    fn custom_acronyms_flag() {
        assert_eq!(
            Clean
                .apply("use the tui", &args(&["--acronyms", "tui"]))
                .unwrap(),
            "Use the TUI\n"
        );
    }

    #[test]
    fn respects_capitalized_words_by_default() {
        // ALL-CAPS words are left alone unless --no-respect-caps is given.
        assert_eq!(
            Clean.apply("THIS is LOUD", &args(&[])).unwrap(),
            "THIS is LOUD\n"
        );
    }

    #[test]
    fn no_respect_caps_folds_shouting() {
        // De-shouted, then the sentence start is re-capitalized for grammar.
        assert_eq!(
            Clean
                .apply("THIS is LOUD", &args(&["--no-respect-caps"]))
                .unwrap(),
            "This is loud\n"
        );
    }

    #[test]
    fn trailing_punctuation_stripped_with_flag() {
        assert_eq!(
            Clean
                .apply("Hello world.", &args(&["--no-trailing-punctuation"]))
                .unwrap(),
            "Hello world\n"
        );
        assert_eq!(
            Clean
                .apply("Wait, really?!", &args(&["--no-trailing-punctuation"]))
                .unwrap(),
            "Wait, really\n"
        );
    }

    #[test]
    fn collapses_blank_line_runs() {
        // Only the start-of-text word is capitalized; the second line has no
        // preceding sentence-ending punctuation, so it stays lowercase.
        assert_eq!(
            Clean.apply("go\n\n\n\ngo\n", &args(&[])).unwrap(),
            "Go\n\ngo\n"
        );
    }

    #[test]
    fn keep_blank_lines_flag() {
        assert_eq!(
            Clean
                .apply("go\n\n\n\ngo", &args(&["--keep-blank-lines"]))
                .unwrap(),
            "Go\n\n\n\ngo\n"
        );
    }

    #[test]
    fn sentence_casing_does_not_capitalize_wrapped_lines() {
        // A sentence wrapped across lines keeps the continuation lowercase…
        assert_eq!(
            Clean
                .apply("the quick brown\nfox jumps", &args(&[]))
                .unwrap(),
            "The quick brown\nfox jumps\n"
        );
        // …but a real sentence break (period) still capitalizes across the line.
        assert_eq!(
            Clean.apply("one thing.\ntwo things", &args(&[])).unwrap(),
            "One thing.\nTwo things\n"
        );
    }

    #[test]
    fn trims_surrounding_blank_lines_but_keeps_indentation() {
        assert_eq!(Clean.apply("\n\n  hi  \n\n", &args(&[])).unwrap(), "  Hi\n");
    }

    #[test]
    fn squeeze_preserves_leading_indentation() {
        assert_eq!(
            Clean.apply("    go    now", &args(&[])).unwrap(),
            "    Go now\n"
        );
    }

    #[test]
    fn removes_zero_width_and_control_chars() {
        let input = "a\u{200B}b\u{0007}c";
        assert_eq!(Clean.apply(input, &args(&[])).unwrap(), "Abc\n");
    }

    #[test]
    fn ascii_flag_folds_smart_punctuation() {
        let input = "\u{201C}quote\u{201D} \u{2014} it\u{2019}s fine\u{2026}";
        assert_eq!(
            Clean.apply(input, &args(&["--ascii"])).unwrap(),
            "\"Quote\" - it's fine...\n"
        );
    }

    #[test]
    fn ascii_flag_replaces_nbsp_with_space() {
        assert_eq!(
            Clean
                .apply("go\u{00A0}\u{00A0}now", &args(&["--ascii"]))
                .unwrap(),
            "Go now\n"
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
            "Hi"
        );
    }
}
