//! `spellcheck` — correct spelling via the system checker while preserving the
//! document's structure.
//!
//! Correction is deliberately conservative and structure-preserving:
//!
//! * Each line is processed independently, so newlines are never moved.
//! * Within a line, only the alphabetic core of isolated words is replaced, so
//!   the exact whitespace between words — including tabs and indentation — and
//!   any surrounding markup (markdown markers, punctuation, quotes) is left
//!   untouched.
//!
//! Spell checking relies on macOS's `NSSpellChecker`; on other platforms the
//! command reports that it is unsupported rather than silently passing text
//! through unchanged.

use crate::casing::segment;
use crate::spellcheck::SpellChecker;
use crate::transform::Transform;
use clap::ArgMatches;

/// Spelling correction using the system spell checker.
pub struct Spellcheck;

impl Spellcheck {
    /// Correct each isolated word on a line, preserving the whitespace between
    /// them exactly.
    fn correct_line_spelling(line: &str, checker: &SpellChecker) -> Result<String, String> {
        segment(line)
            .into_iter()
            .map(|seg| {
                if seg.is_word {
                    checker.correct_word(&seg.text)
                } else {
                    Ok(seg.text)
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|segments| segments.concat())
    }
}

impl Transform for Spellcheck {
    fn name(&self) -> &'static str {
        "spellcheck"
    }

    fn about(&self) -> &'static str {
        "Fix spelling with the system spell checker [e.g. \"recieve the mesage\" -> \"receive the message\"]"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Correct misspelled words using the system spell checker, replacing \
             each misspelling with its first suggested correction. The structure \
             of the text is preserved: line breaks, tabs, indentation, and \
             surrounding markup (markdown markers, punctuation, quotes) are left \
             untouched — only the words themselves are changed. Requires macOS.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["spell"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        let checker = SpellChecker::new()?;

        // Split on '\n' and rejoin so every line boundary — including a trailing
        // newline — is reproduced exactly.
        input
            .split('\n')
            .map(|line| Self::correct_line_spelling(line, &checker))
            .collect::<Result<Vec<_>, String>>()
            .map(|lines| lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    /// Empty argument matches; `spellcheck` declares no flags of its own.
    fn args(_extra: &[&str]) -> ArgMatches {
        Command::new("spellcheck").get_matches_from(["spellcheck"])
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn reports_unsupported_off_macos() {
        assert!(Spellcheck.apply("teh", &args(&[])).is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn corrects_isolated_misspellings() {
        assert_eq!(
            Spellcheck
                .apply("i recieve the mesage", &args(&[]))
                .unwrap(),
            "i receive the message"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn preserves_line_structure_and_indentation() {
        // Tabs, indentation, blank lines, and the trailing newline survive.
        let input = "recieve cat\n\n\tmesage dog\n";
        assert_eq!(
            Spellcheck.apply(input, &args(&[])).unwrap(),
            "receive cat\n\n\tmessage dog\n"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn preserves_markdown_markers() {
        // Only the word core is corrected; the markdown markers stay put.
        assert_eq!(
            Spellcheck.apply("- **mesage** item", &args(&[])).unwrap(),
            "- **message** item"
        );
    }
}
