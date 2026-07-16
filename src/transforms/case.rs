//! Simple whole-string case conversions.
//!
//! These are intentionally tiny — they double as the reference example for how
//! little code a new [`Transform`] requires.

use crate::transform::Transform;
use clap::ArgMatches;

/// Convert all cased characters to uppercase.
pub struct Upper;

impl Transform for Upper {
    fn name(&self) -> &'static str {
        "upper"
    }

    fn about(&self) -> &'static str {
        "Convert text to UPPERCASE"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["uc"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(input.to_uppercase())
    }
}

/// Convert all cased characters to lowercase.
pub struct Lower;

impl Transform for Lower {
    fn name(&self) -> &'static str {
        "lower"
    }

    fn about(&self) -> &'static str {
        "Convert text to lowercase"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["lc"]
    }

    fn apply(&self, input: &str, _args: &ArgMatches) -> Result<String, String> {
        Ok(input.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    /// Empty argument matches, sufficient for transforms with no flags.
    fn no_args() -> ArgMatches {
        Command::new("t").get_matches_from(["t"])
    }

    #[test]
    fn upper_uppercases() {
        assert_eq!(
            Upper.apply("Hello, World!", &no_args()).unwrap(),
            "HELLO, WORLD!"
        );
    }

    #[test]
    fn upper_handles_unicode() {
        assert_eq!(Upper.apply("stra\u{00df}e", &no_args()).unwrap(), "STRASSE");
    }

    #[test]
    fn lower_lowercases() {
        assert_eq!(
            Lower.apply("Hello, WORLD!", &no_args()).unwrap(),
            "hello, world!"
        );
    }

    #[test]
    fn empty_input_is_preserved() {
        assert_eq!(Upper.apply("", &no_args()).unwrap(), "");
        assert_eq!(Lower.apply("", &no_args()).unwrap(), "");
    }
}
