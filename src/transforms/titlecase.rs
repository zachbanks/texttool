//! `titlecase` — smart title casing (the former `smarttitlecase` script).
//!
//! "Smart" means it applies the conventions people actually expect from titles
//! rather than naively capitalizing every word:
//!
//! - Minor words (articles, short conjunctions/prepositions) stay lowercase…
//! - …unless they are the first or last word of a line, or start a subtitle
//!   (the word right after a colon), which are always capitalized.
//! - Already-capitalized words are respected by default, so acronyms, brand
//!   names, and words the writer deliberately capitalized survive (`NASA`,
//!   `iPhone`, `In`). `--no-respect-caps` turns that off and applies the rules
//!   strictly.
//! - Hyphenated compounds capitalize each part (`Two-Cities`).
//! - Leading and trailing whitespace is stripped; interior spacing is kept.

use crate::casing::{AcronymSet, capitalize_first_alpha, core, has_uppercase};
use crate::transform::Transform;
use clap::{Arg, ArgAction, ArgMatches, Command};

/// Words kept lowercase unless positional rules force capitalization.
const SMALL_WORDS: &[&str] = &[
    "a", "an", "and", "as", "at", "but", "by", "en", "for", "if", "in", "nor", "of", "on", "or",
    "per", "the", "to", "v", "v.", "vs", "vs.", "via",
];

/// Smart title-case conversion.
pub struct TitleCase;

/// Capitalize each hyphen-separated part of a word.
fn capitalize_hyphenated(word: &str) -> String {
    word.split('-')
        .map(capitalize_first_alpha)
        .collect::<Vec<_>>()
        .join("-")
}

/// Apply the casing rules to a single word given its position in the line.
fn cap_word(
    word: &str,
    is_first: bool,
    is_last: bool,
    after_colon: bool,
    respect_caps: bool,
    acronyms: &AcronymSet,
) -> String {
    if word.is_empty() {
        return String::new();
    }
    // Respect words the writer already capitalized (acronyms, brands, names).
    if respect_caps && has_uppercase(word) {
        return word.to_string();
    }
    // Recognized acronyms are fully capitalized regardless of position.
    if acronyms.matches(word) {
        return word.to_uppercase();
    }
    let lower = word.to_lowercase();
    let is_minor = SMALL_WORDS.contains(&core(&lower));
    if is_minor && !is_first && !is_last && !after_colon {
        return lower;
    }
    capitalize_hyphenated(&lower)
}

/// Title-case one line: strip its edges, then case each word in place.
fn titlecase_line(line: &str, respect_caps: bool, acronyms: &AcronymSet) -> String {
    let mut segments = crate::casing::segment(line.trim());
    let word_indices: Vec<usize> = segments
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is_word)
        .map(|(i, _)| i)
        .collect();

    let first = word_indices.first().copied();
    let last = word_indices.last().copied();

    let mut prev_word_ended_colon = false;
    for (i, segment) in segments.iter_mut().enumerate() {
        if !segment.is_word {
            continue;
        }
        let is_first = Some(i) == first;
        let is_last = Some(i) == last;
        let cased = cap_word(
            &segment.text,
            is_first,
            is_last,
            prev_word_ended_colon,
            respect_caps,
            acronyms,
        );
        prev_word_ended_colon = segment.text.ends_with(':');
        segment.text = cased;
    }

    segments.into_iter().map(|s| s.text).collect()
}

/// Build the acronym set from the shared `--acronyms` / `--no-acronyms` flags.
fn acronyms_from_args(args: &ArgMatches) -> AcronymSet {
    let extra: Vec<String> = args
        .get_many::<String>("acronyms")
        .map(|values| values.cloned().collect())
        .unwrap_or_default();
    AcronymSet::new(!args.get_flag("no-acronyms"), &extra)
}

impl Transform for TitleCase {
    fn name(&self) -> &'static str {
        "titlecase"
    }

    fn about(&self) -> &'static str {
        "Convert text to smart Title Case"
    }

    fn long_about(&self) -> Option<&'static str> {
        Some(
            "Convert text to title case using common style rules: minor words \
             (a, an, the, of, to, …) stay lowercase unless they are the first or \
             last word or begin a subtitle after a colon; already-capitalized \
             words (NASA, iPhone, In) are respected unless --no-respect-caps is \
             given; hyphenated compounds are capitalized part-by-part; leading \
             and trailing whitespace is stripped while interior spacing and line \
             breaks are kept.",
        )
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["title", "tc"]
    }

    fn augment(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("no-respect-caps")
                .long("no-respect-caps")
                .help("Re-case already-capitalized words [e.g. \"iPhone\" -> \"Iphone\"]")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("acronyms")
                .long("acronyms")
                .value_name("LIST")
                .help("Extra acronyms to fully capitalize [e.g. --acronyms tui,repl: \"tui\" -> \"TUI\"]")
                .value_delimiter(',')
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("no-acronyms")
                .long("no-acronyms")
                .help("Disable acronym capitalization [e.g. \"api\" -> \"Api\"]")
                .action(ArgAction::SetTrue),
        )
    }

    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String> {
        let respect_caps = !args.get_flag("no-respect-caps");
        let acronyms = acronyms_from_args(args);
        Ok(input
            .split('\n')
            .map(|line| titlecase_line(line, respect_caps, &acronyms))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(extra: &[&str]) -> ArgMatches {
        let cmd = TitleCase.augment(Command::new("titlecase"));
        let mut argv = vec!["titlecase"];
        argv.extend_from_slice(extra);
        cmd.get_matches_from(argv)
    }

    fn tc(input: &str) -> String {
        TitleCase.apply(input, &args(&[])).unwrap()
    }

    #[test]
    fn basic_title() {
        assert_eq!(tc("hello world"), "Hello World");
    }

    #[test]
    fn minor_words_stay_lowercase() {
        assert_eq!(tc("of mice and men"), "Of Mice and Men");
    }

    #[test]
    fn first_and_last_words_always_capitalized() {
        assert_eq!(tc("the end of the war"), "The End of the War");
        assert_eq!(tc("a room with a view"), "A Room With a View");
    }

    #[test]
    fn subtitle_after_colon_is_capitalized() {
        assert_eq!(
            tc("chapter one: the beginning"),
            "Chapter One: The Beginning"
        );
    }

    #[test]
    fn respects_acronyms_and_brands_by_default() {
        assert_eq!(tc("the FBI files"), "The FBI Files");
        assert_eq!(tc("the iPhone era"), "The iPhone Era");
    }

    #[test]
    fn respects_deliberately_capitalized_words() {
        // A capitalized minor word ("In") is kept rather than lowercased.
        assert_eq!(tc("the Cat In the Hat"), "The Cat In the Hat");
    }

    #[test]
    fn capitalizes_known_acronyms() {
        assert_eq!(tc("the nasa api docs"), "The NASA API Docs");
    }

    #[test]
    fn no_acronyms_flag_disables_them() {
        assert_eq!(
            TitleCase
                .apply("the api docs", &args(&["--no-acronyms"]))
                .unwrap(),
            "The Api Docs"
        );
    }

    #[test]
    fn custom_acronyms_flag() {
        assert_eq!(
            TitleCase
                .apply("the tui repl", &args(&["--acronyms", "tui,repl"]))
                .unwrap(),
            "The TUI REPL"
        );
    }

    #[test]
    fn no_respect_caps_recases_everything() {
        assert_eq!(
            TitleCase
                .apply("the iPhone ERA", &args(&["--no-respect-caps"]))
                .unwrap(),
            "The Iphone Era"
        );
    }

    #[test]
    fn hyphenated_words_capitalize_each_part() {
        assert_eq!(tc("a tale of two-cities"), "A Tale of Two-Cities");
        assert_eq!(tc("state-of-the-art design"), "State-Of-The-Art Design");
    }

    #[test]
    fn strips_leading_and_trailing_whitespace() {
        assert_eq!(tc("  hello world  "), "Hello World");
        assert_eq!(tc("\tthe quick brown fox \t"), "The Quick Brown Fox");
    }

    #[test]
    fn preserves_interior_spacing() {
        assert_eq!(tc("a  b"), "A  B");
    }

    #[test]
    fn processes_each_line_independently() {
        assert_eq!(tc("  line one \n line two "), "Line One\nLine Two");
    }

    #[test]
    fn preserves_trailing_newline() {
        assert_eq!(tc("hello world\n"), "Hello World\n");
    }

    #[test]
    fn keeps_apostrophes() {
        assert_eq!(tc("it's a wonderful life"), "It's a Wonderful Life");
    }

    #[test]
    fn full_example() {
        assert_eq!(
            tc("the quick brown fox: a tale of two-cities via the WEB"),
            "The Quick Brown Fox: A Tale of Two-Cities via the WEB"
        );
    }
}
