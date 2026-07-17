//! Shared word-splitting used by the identifier-style case transforms
//! (`camel`, `pascal`, `snake`, `kebab`, `constant`).
//!
//! Splitting a phrase into its component words is the hard part all of those
//! operations share, so it lives here once. The splitter understands three
//! kinds of boundary:
//!
//! - **delimiters** — any non-alphanumeric run (spaces, `_`, `-`, `.`, …);
//! - **case changes** — a lowercase/digit followed by an uppercase
//!   (`helloWorld` → `hello`, `World`);
//! - **acronym ends** — an uppercase run whose last letter starts a new
//!   lowercase word (`HTMLParser` → `HTML`, `Parser`).

/// Split a string into its component words.
///
/// Digits stay attached to the surrounding word (`utf8` stays `utf8`).
pub fn split_words(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();

    for i in 0..chars.len() {
        let c = chars[i];

        if !c.is_alphanumeric() {
            // Delimiter: end the current word (if any) and skip the char.
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }

        if !current.is_empty() {
            let prev = chars[i - 1];
            let case_change = (prev.is_lowercase() || prev.is_ascii_digit()) && c.is_uppercase();
            let acronym_end = prev.is_uppercase()
                && c.is_uppercase()
                && chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if case_change || acronym_end {
                words.push(std::mem::take(&mut current));
            }
        }

        current.push(c);
    }

    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Uppercase the first character and lowercase the rest.
pub fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// A run of a line that is either all whitespace or all non-whitespace.
///
/// Used by transforms that need to rewrite individual words while preserving
/// the exact spacing between them.
pub struct Segment {
    /// The literal text of this run.
    pub text: String,
    /// True when the run is a word (non-whitespace), false for whitespace.
    pub is_word: bool,
}

/// Split a line into alternating word / whitespace [`Segment`]s so the original
/// spacing can be reconstructed exactly by concatenating the pieces.
pub fn segment(line: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_is_word: Option<bool> = None;

    for c in line.chars() {
        let is_word = !c.is_whitespace();
        match current_is_word {
            Some(prev) if prev == is_word => current.push(c),
            Some(prev) => {
                segments.push(Segment {
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
        segments.push(Segment {
            text: current,
            is_word,
        });
    }
    segments
}

/// The alphanumeric core of a word, ignoring surrounding punctuation
/// (`"of,"` -> `"of"`, `"(a)"` -> `"a"`).
pub fn core(word: &str) -> &str {
    word.trim_matches(|c: char| !c.is_alphanumeric())
}

/// Uppercase the first alphabetic character, leaving everything else untouched.
///
/// Unlike [`capitalize`], the remainder of the word keeps its original case, so
/// leading punctuation and already-cased tails are preserved.
pub fn capitalize_first_alpha(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    let mut done = false;
    for c in word.chars() {
        if !done && c.is_alphabetic() {
            result.extend(c.to_uppercase());
            done = true;
        } else {
            result.push(c);
        }
    }
    result
}

/// True if the word's core is a single alphabetic character (`"a"`, `"(I)"`).
pub fn is_single_letter(word: &str) -> bool {
    let c = core(word);
    c.chars().count() == 1 && c.chars().all(char::is_alphabetic)
}

/// True if the word contains any uppercase letter, i.e. it is already
/// capitalized in some way (`"In"`, `"iPhone"`, `"NASA"`).
pub fn has_uppercase(word: &str) -> bool {
    word.chars().any(char::is_uppercase)
}

/// True if the word's core is two-or-more letters and entirely uppercase
/// (`"NASA"`, `"LOUD"`) — i.e. a "shouting" all-caps word.
pub fn is_all_caps(word: &str) -> bool {
    let c = core(word);
    c.chars().count() >= 2
        && c.chars().any(char::is_alphabetic)
        && !c.chars().any(char::is_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_delimiters() {
        assert_eq!(split_words("foo_bar-baz qux"), ["foo", "bar", "baz", "qux"]);
    }

    #[test]
    fn splits_on_case_change() {
        assert_eq!(split_words("helloWorld"), ["hello", "World"]);
    }

    #[test]
    fn splits_acronym_boundaries() {
        assert_eq!(split_words("HTMLParser"), ["HTML", "Parser"]);
        assert_eq!(split_words("getHTTPResponse"), ["get", "HTTP", "Response"]);
    }

    #[test]
    fn keeps_digits_attached() {
        assert_eq!(split_words("utf8 value"), ["utf8", "value"]);
    }

    #[test]
    fn splits_digit_to_upper() {
        assert_eq!(split_words("version2Point"), ["version2", "Point"]);
    }

    #[test]
    fn empty_and_delimiter_only() {
        assert!(split_words("").is_empty());
        assert!(split_words("  __  ").is_empty());
    }

    #[test]
    fn capitalize_basics() {
        assert_eq!(capitalize("hELLO"), "Hello");
        assert_eq!(capitalize("x"), "X");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn segment_preserves_spacing() {
        let joined: String = segment("a  b c").into_iter().map(|s| s.text).collect();
        assert_eq!(joined, "a  b c");
        assert_eq!(segment("a  b").iter().filter(|s| s.is_word).count(), 2);
    }

    #[test]
    fn core_strips_edge_punctuation() {
        assert_eq!(core("of,"), "of");
        assert_eq!(core("(a)"), "a");
        assert_eq!(core("hello"), "hello");
    }

    #[test]
    fn capitalize_first_alpha_keeps_tail_and_punct() {
        assert_eq!(capitalize_first_alpha("(hello)"), "(Hello)");
        assert_eq!(capitalize_first_alpha("iPhone"), "IPhone");
        assert_eq!(capitalize_first_alpha("a"), "A");
    }

    #[test]
    fn single_letter_detection() {
        assert!(is_single_letter("a"));
        assert!(is_single_letter("(I)"));
        assert!(!is_single_letter("ab"));
        assert!(!is_single_letter("1"));
    }

    #[test]
    fn uppercase_predicates() {
        assert!(has_uppercase("In"));
        assert!(!has_uppercase("in"));
        assert!(is_all_caps("NASA"));
        assert!(!is_all_caps("Nasa"));
        assert!(!is_all_caps("I")); // single letter is not "shouting"
    }
}
