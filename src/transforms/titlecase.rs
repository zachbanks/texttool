//! `titlecase` — smart title casing (the former `smarttitlecase` script).
//!
//! "Smart" means it applies the conventions people actually expect from titles
//! rather than naively capitalizing every word:
//!
//! - Minor words (articles, short conjunctions/prepositions) stay lowercase…
//! - …unless they are the first or last word of a line, or start a subtitle
//!   (the word right after a colon), which are always capitalized.
//! - Words that already contain "internal" capitals are left untouched, so
//!   acronyms and brand names survive (`NASA`, `iPhone`, `McKinsey`).
//! - Hyphenated compounds capitalize each part (`Two-Cities`).
//! - Original spacing and line breaks are preserved exactly.

use crate::transform::Transform;
use clap::ArgMatches;

/// Words kept lowercase unless positional rules force capitalization.
const SMALL_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "but", "by", "en", "for", "if", "in", "nor", "of", "on", "or",
    "per", "the", "to", "v", "v.", "vs", "vs.", "via",
];

/// Smart title-case conversion.
pub struct TitleCase;

/// A run of text that is either all whitespace or all non-whitespace.
struct Token {
    text: String,
    is_word: bool,
}

/// Split a line into alternating word / whitespace tokens, preserving both so
/// the original spacing can be reconstructed exactly.
fn tokenize(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut current_is_word: Option<bool> = None;

    for c in line.chars() {
        let is_word = !c.is_whitespace();
        match current_is_word {
            Some(prev) if prev == is_word => current.push(c),
            Some(prev) => {
                tokens.push(Token {
                    text: std::mem::take(&mut current),
                    is_word: prev,
                });
                current.push(c);
                current_is_word = Some(is_word);
            }
            None => {
                current.push(c);
                current_is_word = Some(is_word);
            }
        }
    }
    if let Some(is_word) = current_is_word {
        tokens.push(Token {
            text: current,
            is_word,
        });
    }
    tokens
}

/// True if the word carries capitals beyond the first character, which marks it
/// as an intentional acronym or brand name to leave alone.
fn has_internal_caps(word: &str) -> bool {
    word.chars().skip(1).any(|c| c.is_uppercase())
}

/// The alphanumeric core of a word, ignoring surrounding punctuation, used for
/// minor-word lookup (e.g. `"of,"` -> `"of"`).
fn core(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_string()
}

/// Uppercase the first alphabetic character, leaving the rest as-is.
fn capitalize_first_alpha(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut done = false;
    for c in s.chars() {
        if !done && c.is_alphabetic() {
            result.extend(c.to_uppercase());
            done = true;
        } else {
            result.push(c);
        }
    }
    result
}

/// Capitalize each hyphen-separated part of a word.
fn capitalize_hyphenated(word: &str) -> String {
    word.split('-')
        .map(capitalize_first_alpha)
        .collect::<Vec<_>>()
        .join("-")
}

/// Apply the casing rules to a single word given its position in the line.
fn cap_word(word: &str, is_first: bool, is_last: bool, after_colon: bool) -> String {
    if word.is_empty() {
        return String::new();
    }
    if has_internal_caps(word) {
        return word.to_string();
    }
    let lower = word.to_lowercase();
    let is_minor = SMALL_WORDS.contains(&core(&lower).as_str());
    if is_minor && !is_first && !is_last && !after_colon {
        return lower;
    }
    capitalize_hyphenated(&lower)
}

/// Title-case one line, preserving its internal spacing.
fn titlecase_line(line: &str) -> String {
    let mut tokens = tokenize(line);
    let word_indices: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.is_word)
        .map(|(i, _)| i)
        .collect();

    let first = word_indices.first().copied();
    let last = word_indices.last().copied();

    let mut prev_word_ended_colon = false;
    for (i, token) in tokens.iter_mut().enumerate() {
        if !token.is_word {
            continue;
        }
        let is_first = Some(i) == first;
        let is_last = Some(i) == last;
        let cased = cap_word(&token.text, is_first, is_last, prev_word_ended_colon);
        prev_word_ended_colon = token.text.ends_with(':');
        token.text = cased;
    }

    tokens.into_iter().map(|t| t.text).collect()
}

impl Transform for TitleCase {
    fn name(&self) -> &'static str {
        "titlecase"
    }

    fn about(&self) -> &'static str {
        "Convert text to smart Title Case"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Convert text to title case using common style rules: minor words \
             (a, an, the, of, to, …) stay lowercase unless they are the first or \
             last word or begin a subtitle after a colon; acronyms and brand \
             names with internal capitals (NASA, iPhone) are preserved; \
             hyphenated compounds are capitalized part-by-part; and existing \
             spacing and line breaks are kept intact.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["title", "tc"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(input
            .split('\n')
            .map(titlecase_line)
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

    fn tc(input: &str) -> String {
        TitleCase.apply(input, &no_args()).unwrap()
    }

    #[test]
    fn basic_title() {
        assert_eq!(tc("hello world"), "Hello World");
    }

    #[test]
    fn minor_words_stay_lowercase() {
        assert_eq!(tc("of mice and men"), "Of Mice and Men");
    }

    #[test]
    fn first_and_last_words_always_capitalized() {
        // Leading and trailing minor words are still capitalized.
        assert_eq!(tc("the end of the war"), "The End of the War");
        assert_eq!(tc("a room with a view"), "A Room With a View");
    }

    #[test]
    fn subtitle_after_colon_is_capitalized() {
        assert_eq!(
            tc("chapter one: the beginning"),
            "Chapter One: The Beginning"
        );
    }

    #[test]
    fn preserves_acronyms_and_brands() {
        assert_eq!(tc("the FBI files"), "The FBI Files");
        assert_eq!(tc("the iPhone era"), "The iPhone Era");
    }

    #[test]
    fn hyphenated_words_capitalize_each_part() {
        assert_eq!(tc("a tale of two-cities"), "A Tale of Two-Cities");
        assert_eq!(tc("state-of-the-art design"), "State-Of-The-Art Design");
    }

    #[test]
    fn preserves_internal_spacing() {
        assert_eq!(tc("a  b"), "A  B");
    }

    #[test]
    fn processes_each_line_independently() {
        assert_eq!(tc("line one\nline two"), "Line One\nLine Two");
    }

    #[test]
    fn preserves_trailing_newline() {
        assert_eq!(tc("hello world\n"), "Hello World\n");
    }

    #[test]
    fn keeps_apostrophes() {
        assert_eq!(tc("it's a wonderful life"), "It's a Wonderful Life");
    }

    #[test]
    fn full_example() {
        assert_eq!(
            tc("the quick brown fox: a tale of two-cities via the WEB"),
            "The Quick Brown Fox: A Tale of Two-Cities via the WEB"
        );
    }
}
