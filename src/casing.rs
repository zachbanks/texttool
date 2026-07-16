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
}
