//! `humanize` — turn a filename or slug into clean, readable text.
//!
//! A convenience pipeline that composes existing transforms:
//!
//! 1. Drop a trailing file extension (`.pdf`, `.jpeg`, …).
//! 2. [`Unslug`] — split on every common filename delimiter (`-`, `_`, `.`,
//!    spaces, …) and `camelCase`/`ACRONYM` boundaries into spaced words.
//! 3. [`Clean`] — tidy whitespace and fix casing (sentence starts, acronyms,
//!    single letters).
//!
//! `Thorne-magnesium-receipt-2026-07-17.pdf` -> `Thorne magnesium receipt 2026 07 17`

use crate::transform::Transform;
use crate::transforms::{Clean, TitleCase, Unslug};
use clap::{Arg, ArgAction, ArgMatches, Command};
use regex::Regex;

/// Filename/slug → readable text.
pub struct Humanize;

/// Default (flag-free) `ArgMatches` for a transform, so it can be invoked
/// programmatically with all of its flags at their defaults.
fn default_matches(name: &'static str, augment: impl Fn(Command) -> Command) -> ArgMatches {
    augment(Command::new(name)).get_matches_from([name])
}

impl Transform for Humanize {
    fn name(&self) -> &'static str {
        "humanize"
    }

    fn about(&self) -> &'static str {
        "Turn a filename/slug into clean, readable text"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Turn a filename or slug into readable text: drop a trailing file \
             extension, split on every common filename delimiter (-, _, ., \
             spaces) and camelCase/ACRONYM boundaries, then clean up whitespace \
             and casing. Composes `unslug` and `clean`.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["readable"]
    }

    fn augment(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .help("Title Case each word instead of sentence casing [e.g. \"my file\" -> \"My File\"]")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        // 1. Drop a trailing file extension on each line (.pdf, .jpeg, …).
        let ext = Regex::new(r"(?m)\.[A-Za-z0-9]{1,6}$").expect("valid regex");
        let no_ext = ext.replace_all(input, "").into_owned();

        // 2. Unslug: all delimiters + camelCase boundaries -> spaced words.
        let unslug_args = default_matches("unslug", |c| Unslug.augment(c));
        let spaced = Unslug.apply(&no_ext, &unslug_args)?;

        // 3. Case it: Title Case each word (--title) or clean's sentence casing.
        if args.get_flag("title") {
            let title_args = default_matches("titlecase", |c| TitleCase.augment(c));
            TitleCase.apply(&spaced, &title_args)
        } else {
            let clean_args = default_matches("clean", |c| Clean.augment(c));
            Clean.apply(&spaced, &clean_args)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Humanize.augment(Command::new("humanize"));
        let mut argv = vec!["humanize"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    fn humanize(input: &str) -> String {
        Humanize.apply(input, &args(&[])).unwrap()
    }

    #[test]
    fn filename_to_readable() {
        assert_eq!(
            humanize("Thorne-magnesium-trumed-receipt-2026-07-17.pdf"),
            "Thorne magnesium trumed receipt 2026 07 17\n"
        );
    }

    #[test]
    fn strips_extension_and_mixed_delimiters() {
        assert_eq!(humanize("my_vacation.photo.JPG"), "My vacation photo\n");
    }

    #[test]
    fn capitalizes_sentence_start_and_acronyms() {
        assert_eq!(humanize("annual_api_report.docx"), "Annual API report\n");
    }

    #[test]
    fn no_extension_is_fine() {
        assert_eq!(humanize("first-draft"), "First draft\n");
    }

    #[test]
    fn title_flag_title_cases_each_word() {
        assert_eq!(
            Humanize
                .apply("my_vacation.photo.JPG", &args(&["--title"]))
                .unwrap(),
            "My Vacation Photo"
        );
        // Acronyms still capitalized; minor words stay lowercase.
        assert_eq!(
            Humanize
                .apply("annual_api_report_of_the_year.docx", &args(&["--title"]))
                .unwrap(),
            "Annual API Report of the Year"
        );
    }
}
