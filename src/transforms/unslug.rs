//! `unslug` — turn slugs and identifiers back into spaced words.
//!
//! The inverse of `slug` (and a de-caser for identifier styles): it splits on
//! `-`, `_`, `.`, other separators, and `camelCase`/`ACRONYM` boundaries, then
//! joins the words with single spaces. Original word casing is preserved.

use crate::casing::split_words;
use crate::transform::Transform;
use clap::ArgMatches;

/// Split slugs/identifiers into spaced words.
pub struct Unslug;

impl Transform for Unslug {
    fn name(&self) -> &'static str {
        "unslug"
    }

    fn about(&self) -> &'static str {
        "Turn slugs/identifiers into spaced words (inverse of slug) [e.g. \"helloWorld\" -> \"hello World\"]"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Split each line on separators (-, _, ., …) and camelCase/ACRONYM \
             boundaries, then join the words with single spaces. The inverse of \
             slug; word casing is preserved (e.g. `Thorne-magnesium-2026` -> \
             `Thorne magnesium 2026`, `getHTTPResponse` -> `get HTTP Response`).",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["deslug"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(input
            .split('\n')
            .map(|line| split_words(line).join(" "))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    fn no_args() -> ArgMatches {
        Command::new("t").get_matches_from(["t"])
    }

    fn unslug(input: &str) -> String {
        Unslug.apply(input, &no_args()).unwrap()
    }

    #[test]
    fn hyphens_to_spaces() {
        assert_eq!(
            unslug("Thorne-magnesium-trumed-receipt-2026-07-17"),
            "Thorne magnesium trumed receipt 2026 07 17"
        );
    }

    #[test]
    fn underscores_and_dots() {
        assert_eq!(unslug("hello_world.foo"), "hello world foo");
    }

    #[test]
    fn camel_and_acronyms() {
        assert_eq!(unslug("getHTTPResponse"), "get HTTP Response");
        assert_eq!(unslug("helloWorld"), "hello World");
    }

    #[test]
    fn per_line() {
        assert_eq!(unslug("one-two\nthree_four"), "one two\nthree four");
    }

    #[test]
    fn empty_stays_empty() {
        assert_eq!(unslug(""), "");
        assert_eq!(unslug("---"), "");
    }
}
