//! `mock` — aLtErNaTiNg "mocking SpongeBob" case, jUsT fOr FuN.
//!
//! Alphabetic characters alternate between lower- and uppercase; everything
//! else passes through untouched and does not advance the alternation, so the
//! rhythm survives across spaces and punctuation.

use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Alternating-case conversion.
pub struct Mock;

impl Transform for Mock {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn about(&self) -> &'static str {
        "Convert text to mOcKiNg aLtErNaTiNg case"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["spongebob", "mocking", "sarcasm", "alternate"]
    }

    fn augment(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("start-upper")
                .long("start-upper")
                .help("Start with an uppercase letter [e.g. \"abc\" -> \"AbC\"]")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let start_upper = args.get_flag("start-upper");
        let mut index: usize = 0;
        let mut out = String::with_capacity(input.len());

        for c in input.chars() {
            if c.is_alphabetic() {
                let make_upper = (index % 2 == 1) ^ start_upper;
                if make_upper {
                    out.extend(c.to_uppercase());
                } else {
                    out.extend(c.to_lowercase());
                }
                index += 1;
            } else {
                out.push(c);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = Mock.augment(Command::new("mock"));
        let mut argv = vec!["mock"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    #[test]
    fn alternates_starting_lower() {
        assert_eq!(
            Mock.apply("just for fun", &args(&[])).unwrap(),
            "jUsT fOr FuN"
        );
    }

    #[test]
    fn start_upper_flag() {
        assert_eq!(Mock.apply("abc", &args(&["--start-upper"])).unwrap(), "AbC");
    }

    #[test]
    fn non_letters_do_not_advance_rhythm() {
        // Digits and punctuation pass through and keep their place in the beat.
        assert_eq!(Mock.apply("a1b2c", &args(&[])).unwrap(), "a1B2c");
    }

    #[test]
    fn empty_input() {
        assert_eq!(Mock.apply("", &args(&[])).unwrap(), "");
    }
}
