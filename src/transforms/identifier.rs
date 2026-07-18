//! Identifier-style case conversions built on the shared word splitter:
//! `camel`, `pascal`, `snake`, `kebab`, and `constant`.
//!
//! Each converts every line into a single identifier in the target style, so a
//! list of phrases becomes a list of identifiers.

use crate::casing::{capitalize, split_words};
use crate::transform::Transform;
use clap::ArgMatches;

/// Apply a per-line word-joining function over the whole input.
fn per_line(input: &str, join: impl Fn(Vec<String>) -> String) -> String {
    input
        .split('\n')
        .map(|line| join(split_words(line)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `camelCase`: first word lowercase, the rest capitalized, joined together.
pub struct Camel;

impl Transform for Camel {
    fn name(&self) -> &'static str {
        "camel"
    }

    fn about(&self) -> &'static str {
        "Convert text to camelCase [e.g. \"hello world\" -> \"helloWorld\"]"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["camelcase"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(per_line(input, |words| {
            words
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    if i == 0 {
                        w.to_lowercase()
                    } else {
                        capitalize(w)
                    }
                })
                .collect()
        }))
    }
}

/// `PascalCase`: every word capitalized and joined together.
pub struct Pascal;

impl Transform for Pascal {
    fn name(&self) -> &'static str {
        "pascal"
    }

    fn about(&self) -> &'static str {
        "Convert text to PascalCase [e.g. \"hello world\" -> \"HelloWorld\"]"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["pascalcase", "upper-camel"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(per_line(input, |words| {
            words.iter().map(|w| capitalize(w)).collect()
        }))
    }
}

/// `snake_case`: lowercase words joined with underscores.
pub struct Snake;

impl Transform for Snake {
    fn name(&self) -> &'static str {
        "snake"
    }

    fn about(&self) -> &'static str {
        "Convert text to snake_case [e.g. \"hello world\" -> \"hello_world\"]"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["snakecase"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(per_line(input, |words| {
            words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("_")
        }))
    }
}

/// `kebab-case`: lowercase words joined with hyphens.
pub struct Kebab;

impl Transform for Kebab {
    fn name(&self) -> &'static str {
        "kebab"
    }

    fn about(&self) -> &'static str {
        "Convert text to kebab-case [e.g. \"hello world\" -> \"hello-world\"]"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["kebabcase"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(per_line(input, |words| {
            words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join("-")
        }))
    }
}

/// `CONSTANT_CASE`: uppercase words joined with underscores (SCREAMING_SNAKE).
pub struct Constant;

impl Transform for Constant {
    fn name(&self) -> &'static str {
        "constant"
    }

    fn about(&self) -> &'static str {
        "Convert text to CONSTANT_CASE (SCREAMING_SNAKE_CASE) [e.g. \"hello world\" -> \"HELLO_WORLD\"]"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["scream", "screaming-snake", "const"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(per_line(input, |words| {
            words
                .iter()
                .map(|w| w.to_uppercase())
                .collect::<Vec<_>>()
                .join("_")
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    fn no_args() -> ArgMatches {
        Command::new("t").get_matches_from(["t"])
    }

    fn run(t: &dyn Transform, input: &str) -> String {
        t.apply(input, &no_args()).unwrap()
    }

    #[test]
    fn camel_case() {
        assert_eq!(run(&Camel, "hello world"), "helloWorld");
        assert_eq!(run(&Camel, "Foo_Bar-baz"), "fooBarBaz");
    }

    #[test]
    fn pascal_case() {
        assert_eq!(run(&Pascal, "hello world"), "HelloWorld");
        assert_eq!(run(&Pascal, "HTMLParser value"), "HtmlParserValue");
    }

    #[test]
    fn snake_case() {
        assert_eq!(run(&Snake, "Hello World"), "hello_world");
        assert_eq!(run(&Snake, "getHTTPResponse"), "get_http_response");
    }

    #[test]
    fn kebab_case() {
        assert_eq!(run(&Kebab, "Hello World"), "hello-world");
    }

    #[test]
    fn constant_case() {
        assert_eq!(run(&Constant, "hello world"), "HELLO_WORLD");
        assert_eq!(run(&Constant, "maxRetryCount"), "MAX_RETRY_COUNT");
    }

    #[test]
    fn per_line_conversion() {
        assert_eq!(
            run(&Snake, "One Thing\nTwo Things"),
            "one_thing\ntwo_things"
        );
    }

    #[test]
    fn empty_line_stays_empty() {
        assert_eq!(run(&Camel, ""), "");
        assert_eq!(run(&Constant, "!!!"), "");
    }
}
