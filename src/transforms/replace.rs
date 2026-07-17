//! `replace` — general find-and-replace over the input.
//!
//! Literal by default; `--regex` treats the pattern as a regular expression
//! (with `$1` group references in the replacement). `--ignore-case` matches
//! case-insensitively either way.

use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};
use regex::{NoExpand, Regex};

/// Find-and-replace transform.
pub struct Replace;

impl Transform for Replace {
    fn name(&self) -> &'static str {
        "replace"
    }

    fn about(&self) -> &'static str {
        "Find and replace text (literal or regex)"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Replace all occurrences of FROM with TO. FROM is literal text by \
             default, or a regular expression with --regex (then TO may use $1 \
             group references). --ignore-case matches case-insensitively. Pass an \
             empty TO ('') to delete matches. Reads file operands or stdin as \
             usual.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["sub"]
    }

    fn augment(&self, cmd: Command) -> Command {
        // Declare FROM and TO ahead of the shared FILES operand so they take the
        // first two positional slots; FILES remains the trailing variadic one.
        cmd.arg(
            Arg::new("from")
                .value_name("FROM")
                .required(true)
                .help("Text to find (or a regex with --regex) [e.g. -]"),
        )
        .arg(
            Arg::new("to")
                .value_name("TO")
                .required(true)
                .help("Replacement; '' deletes, $1 refers to groups in regex mode [e.g. ' ']"),
        )
        .arg(
            Arg::new("regex")
                .short('r')
                .long("regex")
                .help("Treat FROM as a regular expression [e.g. -r '[-_.]+' ' ']")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ignore-case")
                .short('i')
                .long("ignore-case")
                .help("Match case-insensitively [e.g. \"a\" matches \"A\"]")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let from = args.get_one::<String>("from").expect("required");
        let to = args.get_one::<String>("to").expect("required");
        let use_regex = args.get_flag("regex");
        let ignore = args.get_flag("ignore-case");

        if use_regex {
            let pattern = if ignore {
                format!("(?i){from}")
            } else {
                from.clone()
            };
            let re = Regex::new(&pattern).map_err(|e| format!("invalid regex `{from}`: {e}"))?;
            Ok(re.replace_all(input, to.as_str()).into_owned())
        } else if ignore {
            // Case-insensitive literal: escape FROM, and keep TO literal so `$`
            // is not treated as a group reference.
            let re = Regex::new(&format!("(?i){}", regex::escape(from)))
                .map_err(|e| format!("failed to build matcher: {e}"))?;
            Ok(re.replace_all(input, NoExpand(to.as_str())).into_owned())
        } else {
            Ok(input.replace(from.as_str(), to))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Replace.augment(Command::new("replace"));
        let mut argv = vec!["replace"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    fn replace(input: &str, extra: &[&str]) -> String {
        Replace.apply(input, &args(extra)).unwrap()
    }

    #[test]
    fn literal_replace() {
        assert_eq!(replace("a-b-c", &["-", " "]), "a b c");
    }

    #[test]
    fn literal_delete_with_empty_to() {
        assert_eq!(replace("a-b-c", &["-", ""]), "abc");
    }

    #[test]
    fn regex_replace_with_groups() {
        assert_eq!(
            replace("2026-07-17", &["-r", r"(\d+)-(\d+)-(\d+)", "$3/$2/$1"]),
            "17/07/2026"
        );
    }

    #[test]
    fn regex_character_class() {
        assert_eq!(replace("a-b_c.d", &["-r", "[-_.]+", " "]), "a b c d");
    }

    #[test]
    fn ignore_case_literal() {
        assert_eq!(replace("Foo foo FOO", &["-i", "foo", "bar"]), "bar bar bar");
    }

    #[test]
    fn ignore_case_literal_keeps_dollar_literal() {
        // In literal mode, `$1` in the replacement stays literal.
        assert_eq!(replace("aXa", &["-i", "x", "$1"]), "a$1a");
    }

    #[test]
    fn invalid_regex_errors() {
        let err = Replace.apply("x", &args(&["-r", "(", "y"]));
        assert!(err.is_err());
    }
}
