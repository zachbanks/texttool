//! `slug` — turn text into URL/filename-friendly slugs.
//!
//! Each line is lowercased, non-alphanumeric runs are replaced with a single
//! separator, and leading/trailing separators are trimmed. Line breaks are
//! preserved so a list of titles becomes a list of slugs.

use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Slugify text (lowercase, separator-joined, alphanumeric only).
pub struct Slug;

impl Slug {
    /// Slugify a single line with the given separator and alphabet policy.
    fn slug_line(line: &str, sep: &str, unicode: bool) -> String {
        let mut out = String::with_capacity(line.len());
        let mut pending_sep = false;

        for c in line.chars() {
            let keep = if unicode {
                c.is_alphanumeric()
            } else {
                c.is_ascii_alphanumeric()
            };

            if keep {
                if pending_sep && !out.is_empty() {
                    out.push_str(sep);
                }
                pending_sep = false;
                out.extend(c.to_lowercase());
            } else {
                // Defer emitting a separator until a kept char follows, so
                // leading and trailing separators never appear.
                pending_sep = true;
            }
        }
        out
    }
}

impl Transform for Slug {
    fn name(&self) -> &'static str {
        "slug"
    }

    fn about(&self) -> &'static str {
        "Slugify text into URL/filename-friendly form [e.g. \"Hello, World!\" -> \"hello-world\"]"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Convert each line into a slug: lowercase it, replace every run of \
             non-alphanumeric characters with a single separator, and trim \
             leading/trailing separators. By default only ASCII letters and \
             digits are kept; use --unicode to keep all Unicode alphanumerics.",
        )
    }

    fn augment(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("sep")
                .short('s')
                .long("sep")
                .help("Separator to join words with [e.g. --sep _: \"a b\" -> \"a_b\"]")
                .value_name("SEP")
                .default_value("-"),
        )
        .arg(
            Arg::new("unicode")
                .long("unicode")
                .help("Keep Unicode alphanumerics [e.g. \"Café\" -> \"café\" not \"caf\"]")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let sep = args
            .get_one::<String>("sep")
            .map(String::as_str)
            .unwrap_or("-");
        let unicode = args.get_flag("unicode");

        Ok(input
            .split('\n')
            .map(|line| Self::slug_line(line, sep, unicode))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Slug.augment(Command::new("slug"));
        let mut argv = vec!["slug"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    #[test]
    fn basic_slug() {
        assert_eq!(
            Slug.apply("Hello, World!", &args(&[])).unwrap(),
            "hello-world"
        );
    }

    #[test]
    fn collapses_and_trims_separators() {
        assert_eq!(
            Slug.apply("  --Foo   & Bar--  ", &args(&[])).unwrap(),
            "foo-bar"
        );
    }

    #[test]
    fn custom_separator() {
        assert_eq!(
            Slug.apply("Foo Bar Baz", &args(&["--sep", "_"])).unwrap(),
            "foo_bar_baz"
        );
    }

    #[test]
    fn ascii_only_drops_non_ascii() {
        // Accented letters are dropped in the default ASCII mode.
        assert_eq!(
            Slug.apply("Café del Mar", &args(&[])).unwrap(),
            "caf-del-mar"
        );
    }

    #[test]
    fn unicode_mode_keeps_alphanumerics() {
        assert_eq!(
            Slug.apply("Café del Mar", &args(&["--unicode"])).unwrap(),
            "café-del-mar"
        );
    }

    #[test]
    fn per_line() {
        assert_eq!(
            Slug.apply("Hello World\nFoo Bar", &args(&[])).unwrap(),
            "hello-world\nfoo-bar"
        );
    }

    #[test]
    fn empty_line_stays_empty() {
        assert_eq!(Slug.apply("!!!", &args(&[])).unwrap(), "");
    }
}
