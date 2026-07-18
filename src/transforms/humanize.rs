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

use crate::patterns::{add_pattern_args, default_patterns, load_categories};
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
        "Turn a filename/slug into clean, readable text [e.g. \"my_vacation.photo.JPG\" -> \"My vacation photo\"]"
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
        let cmd = cmd
            .arg(
                Arg::new("title")
                    .short('t')
                    .long("title")
                    .help("Title Case each word instead of sentence casing [e.g. \"my file\" -> \"My File\"]")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("keep-dates")
                    .long("keep-dates")
                    .help("Keep dates intact, delimiters and all [e.g. \"a-2026-07-17\" -> \"A 2026-07-17\"]")
                    .action(ArgAction::SetTrue),
            );
        // --patterns-file, so the Dates pattern used by --keep-dates is configurable.
        add_pattern_args(cmd)
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        // 0. Optionally shield dates (delimiters and all) behind placeholders
        //    that survive unslug/clean untouched, restored at the very end.
        let mut protected: Vec<String> = Vec::new();
        let mut work = input.to_string();
        if args.get_flag("keep-dates") {
            // Underscore is a regex word char, so `\b` in the Dates pattern won't
            // fire next to one. Normalize `_` -> `-` first (transparent, since
            // unslug treats both as delimiters) so `_`-delimited dates match too.
            work = work.replace('_', "-");
            // Prefer a configured Dates pattern (honoring --patterns-file and any
            // override), but always fall back to the built-in so disabling Dates
            // in config never leaves --keep-dates doing nothing.
            let configured = load_categories(args)?
                .into_iter()
                .find(|c| c.name.eq_ignore_ascii_case("dates"))
                .map(|c| c.regex);
            let dates_regex = configured.unwrap_or_else(|| {
                default_patterns()
                    .into_iter()
                    .find(|c| c.name.eq_ignore_ascii_case("dates"))
                    .map(|c| c.regex)
                    .expect("built-in Dates category always exists")
            });
            let re = Regex::new(&dates_regex).map_err(|e| format!("invalid Dates pattern: {e}"))?;
            work = re
                .replace_all(&work, |caps: &regex::Captures| {
                    let token = format!("DATEHOLDER{}", protected.len());
                    protected.push(caps[0].to_string());
                    token
                })
                .into_owned();
        }

        // 1. Drop a trailing file extension on each line (.pdf, .jpeg, …).
        let ext = Regex::new(r"(?m)\.[A-Za-z0-9]{1,6}$").expect("valid regex");
        let no_ext = ext.replace_all(&work, "").into_owned();

        // 2. Unslug: all delimiters + camelCase boundaries -> spaced words.
        let unslug_args = default_matches("unslug", |c| Unslug.augment(c));
        let spaced = Unslug.apply(&no_ext, &unslug_args)?;

        // 3. Case it: Title Case each word (--title) or clean's sentence casing.
        let mut cased = if args.get_flag("title") {
            let title_args = default_matches("titlecase", |c| TitleCase.augment(c));
            TitleCase.apply(&spaced, &title_args)?
        } else {
            let clean_args = default_matches("clean", |c| Clean.augment(c));
            Clean.apply(&spaced, &clean_args)?
        };

        // 4. Restore protected dates (high index first so DATEHOLDER1 doesn't
        //    match inside DATEHOLDER10).
        for i in (0..protected.len()).rev() {
            cased = cased.replace(&format!("DATEHOLDER{i}"), &protected[i]);
        }
        Ok(cased)
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
    fn keep_dates_preserves_delimiters() {
        assert_eq!(
            Humanize
                .apply("Thorne-receipt-2026-07-17.pdf", &args(&["--keep-dates"]))
                .unwrap(),
            "Thorne receipt 2026-07-17\n"
        );
    }

    #[test]
    fn keep_dates_with_slash_date() {
        assert_eq!(
            Humanize
                .apply("invoice_12/25/2026_final.pdf", &args(&["--keep-dates"]))
                .unwrap(),
            "Invoice 12/25/2026 final\n"
        );
    }

    #[test]
    fn keep_dates_handles_more_formats() {
        // Slash-ISO (was previously unmatched).
        assert_eq!(
            Humanize
                .apply("notes 2026/07/17 draft", &args(&["--keep-dates"]))
                .unwrap(),
            "Notes 2026/07/17 draft\n"
        );
        // "D Month Y".
        assert_eq!(
            Humanize
                .apply("memo 17 July 2026 v1", &args(&["--keep-dates"]))
                .unwrap(),
            "Memo 17 July 2026 v1\n"
        );
    }

    #[test]
    fn keep_dates_falls_back_when_config_disables_dates() {
        use std::io::Write;
        let mut path = std::env::temp_dir();
        path.push(format!("texttool-hz-{}.toml", std::process::id()));
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[[category]]\nname = \"Dates\"\nenabled = false").unwrap();
        let out = Humanize
            .apply(
                "receipt-2026-07-17",
                &args(&["--keep-dates", "--patterns-file", path.to_str().unwrap()]),
            )
            .unwrap();
        let _ = std::fs::remove_file(&path);
        // Built-in Dates pattern still protects the date.
        assert_eq!(out, "Receipt 2026-07-17\n");
    }

    #[test]
    fn without_keep_dates_the_date_is_split() {
        assert_eq!(
            Humanize
                .apply("Thorne-receipt-2026-07-17.pdf", &args(&[]))
                .unwrap(),
            "Thorne receipt 2026 07 17\n"
        );
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
