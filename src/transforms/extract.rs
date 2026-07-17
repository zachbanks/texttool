//! `extract` — pull structured information out of a body of text.
//!
//! Each category (Phone Numbers, Emails, …) is a named regular expression; the
//! set is user-configurable via a TOML file (see [`crate::patterns`]). Matches
//! are grouped under Markdown headings:
//!
//! ```text
//! # Phone Numbers
//! 123-233-1223
//!
//! # Emails
//! jim@example.com
//! ```

use crate::patterns::{Category, add_pattern_args, load_categories};
use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};
use regex::Regex;

/// Information extraction by category.
pub struct Extract;

/// Normalize a name/token for `--only` matching: lowercase, alphanumerics only.
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// True if `token` selects the category `name`.
///
/// Matches the whole name or any of its words, allowing a prefix in either
/// direction so `phones`/`phone` both hit `Phone Numbers` and `ssn` hits `SSNs`.
fn selects(token: &str, name: &str) -> bool {
    let token = normalize(token);
    if token.is_empty() {
        return false;
    }
    let whole = normalize(name);
    if whole == token || whole.starts_with(&token) {
        return true;
    }
    name.split(|c: char| !c.is_alphanumeric())
        .map(normalize)
        .filter(|w| !w.is_empty())
        .any(|w| w == token || w.starts_with(&token) || token.starts_with(&w))
}

impl Extract {
    /// Filter categories by the `--only` list (all categories if `None`).
    fn selected(categories: Vec<Category>, only: Option<&Vec<String>>) -> Vec<Category> {
        let Some(tokens) = only else {
            return categories;
        };
        categories
            .into_iter()
            .filter(|c| tokens.iter().any(|t| selects(t, &c.name)))
            .collect()
    }

    /// Find matches for one category, de-duplicated in first-seen order.
    ///
    /// If the pattern has a capturing group, the first group is reported instead
    /// of the whole match — so a pattern can anchor on a label (e.g.
    /// `Order #\s*(\S+)`) but output only the captured value.
    fn matches(regex: &Regex, input: &str, dedup: bool) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for caps in regex.captures_iter(input) {
            let Some(m) = caps.get(1).or_else(|| caps.get(0)) else {
                continue;
            };
            let value = m.as_str().trim().to_string();
            if value.is_empty() {
                continue;
            }
            if dedup && out.iter().any(|v| v == &value) {
                continue;
            }
            out.push(value);
        }
        out
    }
}

impl Transform for Extract {
    fn name(&self) -> &'static str {
        "extract"
    }

    fn about(&self) -> &'static str {
        "Extract phone numbers, emails, dates, and more into Markdown sections"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Pull structured information out of text using per-category regular \
             expressions, grouped under Markdown headings. Built-in categories \
             (Phone Numbers, Emails, URLs, Addresses, Dates, Times, SSNs, Credit \
             Cards, IP Addresses) can be overridden or extended via a TOML config \
             file; see --patterns-file. If a pattern has a capturing group, only \
             the first group is reported, so a pattern can anchor on a label \
             (e.g. `Order #\\s*(\\S+)`) yet output just the value. Select specific \
             categories with --only and omit headings with --no-headers.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["infoparse"]
    }

    fn augment(&self, cmd: Command) -> Command {
        let cmd = cmd
            .arg(
                Arg::new("only")
                    .long("only")
                    .value_name("LIST")
                    .help("Only these categories [e.g. --only emails,phones]")
                    .value_delimiter(',')
                    .action(ArgAction::Append),
            )
            .arg(
                Arg::new("no-headers")
                    .long("no-headers")
                    .help("Omit the Markdown category headings, print values only")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("show-empty")
                    .long("show-empty")
                    .help("Include categories that had no matches")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-dedup")
                    .long("no-dedup")
                    .help("Keep duplicate matches instead of de-duplicating")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("list")
                    .long("list")
                    .help("List the available categories and their patterns, then exit")
                    .action(ArgAction::SetTrue),
            );
        add_pattern_args(cmd)
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let categories = load_categories(args)?;
        let only = args
            .get_many::<String>("only")
            .map(|v| v.cloned().collect());
        let selected = Self::selected(categories, only.as_ref());

        // Compile up front so a bad pattern reports which category is at fault.
        let compiled: Vec<(String, Regex)> = selected
            .into_iter()
            .map(|c| {
                Regex::new(&c.regex)
                    .map(|re| (c.name.clone(), re))
                    .map_err(|e| format!("invalid regex for `{}`: {e}", c.name))
            })
            .collect::<Result<_, _>>()?;

        if args.get_flag("list") {
            let listing: Vec<String> = compiled
                .iter()
                .map(|(name, re)| format!("{name}\t{}", re.as_str()))
                .collect();
            return Ok(format!("{}\n", listing.join("\n")));
        }

        let headers = !args.get_flag("no-headers");
        let show_empty = args.get_flag("show-empty");
        let dedup = !args.get_flag("no-dedup");

        let mut sections: Vec<String> = Vec::new();
        for (name, regex) in &compiled {
            let values = Self::matches(regex, input, dedup);
            if values.is_empty() && !show_empty {
                continue;
            }
            let body = values.join("\n");
            if headers {
                // Empty (shown) categories are just the heading, no blank body
                // line — the section joiner adds spacing between categories.
                if body.is_empty() {
                    sections.push(format!("# {name}"));
                } else {
                    sections.push(format!("# {name}\n{body}"));
                }
            } else if !body.is_empty() {
                sections.push(body);
            }
        }

        let mut result = sections.join("\n\n");
        if !result.is_empty() {
            result.push('\n');
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Extract.augment(Command::new("extract"));
        let mut argv = vec!["extract"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    fn extract(input: &str, extra: &[&str]) -> String {
        Extract.apply(input, &args(extra)).unwrap()
    }

    const SAMPLE: &str =
        "Call 123-233-1223 or email jim@example.com. Backup: 555-867-5309, jim@example.com again.";

    #[test]
    fn extracts_with_headers() {
        let out = extract(SAMPLE, &[]);
        assert!(out.contains("# Phone Numbers\n123-233-1223"));
        assert!(out.contains("# Emails\njim@example.com"));
    }

    #[test]
    fn dedups_by_default() {
        let out = extract(SAMPLE, &["--only", "emails"]);
        // Two occurrences of the same email collapse to one.
        assert_eq!(out, "# Emails\njim@example.com\n");
    }

    #[test]
    fn no_dedup_keeps_duplicates() {
        let out = extract(SAMPLE, &["--only", "emails", "--no-dedup"]);
        assert_eq!(out, "# Emails\njim@example.com\njim@example.com\n");
    }

    #[test]
    fn only_selects_categories() {
        let out = extract(SAMPLE, &["--only", "phones"]);
        assert!(out.contains("# Phone Numbers"));
        assert!(!out.contains("# Emails"));
    }

    #[test]
    fn no_headers_prints_values_only() {
        let out = extract(SAMPLE, &["--only", "emails", "--no-headers"]);
        assert_eq!(out, "jim@example.com\n");
    }

    #[test]
    fn empty_categories_hidden_unless_requested() {
        let out = extract("no matches here", &["--only", "ssns"]);
        assert_eq!(out, "");
        let shown = extract("no matches here", &["--only", "ssns", "--show-empty"]);
        assert_eq!(shown, "# SSNs\n");
    }

    #[test]
    fn extracts_ssn_and_credit_card() {
        let out = extract("SSN 123-45-6789 card 4111 1111 1111 1111", &[]);
        assert!(out.contains("# SSNs\n123-45-6789"));
        assert!(out.contains("# Credit Cards\n4111 1111 1111 1111"));
    }

    #[test]
    fn list_shows_categories() {
        let out = extract("", &["--list", "--only", "emails"]);
        assert!(out.starts_with("Emails\t"));
    }

    #[test]
    fn capture_group_reported_when_present() {
        // A grouped pattern outputs only the captured value (label anchored).
        let re = Regex::new(r"(?i)order\s*#\s*(\S+)").unwrap();
        assert_eq!(
            Extract::matches(&re, "Order # WK32338412", true),
            vec!["WK32338412"]
        );
        // A pattern with no group still reports the whole match.
        let plain = Regex::new(r"\bJIRA-\d+\b").unwrap();
        assert_eq!(
            Extract::matches(&plain, "see JIRA-42", true),
            vec!["JIRA-42"]
        );
    }

    #[test]
    fn custom_pattern_via_file() {
        use std::io::Write;
        let mut path = std::env::temp_dir();
        path.push(format!("texttool-patterns-{}.toml", std::process::id()));
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "[[category]]\nname = \"Ticket IDs\"\nregex = '''JIRA-\\d+'''"
        )
        .unwrap();
        let out = extract(
            "see JIRA-42 and JIRA-7",
            &[
                "--only",
                "ticket",
                "--patterns-file",
                path.to_str().unwrap(),
            ],
        );
        let _ = std::fs::remove_file(&path);
        assert_eq!(out, "# Ticket IDs\nJIRA-42\nJIRA-7\n");
    }
}
