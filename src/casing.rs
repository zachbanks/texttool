//! Shared word-splitting and casing helpers used across transforms.
//!
//! Splitting a phrase into its component words is the hard part several
//! operations share, so it lives here once. The splitter understands three
//! kinds of boundary:
//!
//! - **delimiters** — any non-alphanumeric run (spaces, `_`, `-`, `.`, …);
//! - **case changes** — a lowercase/digit followed by an uppercase
//!   (`helloWorld` → `hello`, `World`);
//! - **acronym ends** — an uppercase run whose last letter starts a new
//!   lowercase word (`HTMLParser` → `HTML`, `Parser`).
//!
//! It also owns the shared [`AcronymSet`] (known-acronym recognition — the only
//! reliable way to capitalize acronyms, since length alone can't tell `API`
//! from `cat`) and [`capitalize_sentences`] for sentence-start casing.

use std::collections::HashSet;

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

/// A run of a line that is either all whitespace or all non-whitespace.
///
/// Used by transforms that need to rewrite individual words while preserving
/// the exact spacing between them.
pub struct Segment {
    /// The literal text of this run.
    pub text: String,
    /// True when the run is a word (non-whitespace), false for whitespace.
    pub is_word: bool,
}

/// Split a line into alternating word / whitespace [`Segment`]s so the original
/// spacing can be reconstructed exactly by concatenating the pieces.
pub fn segment(line: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_is_word: Option<bool> = None;

    for c in line.chars() {
        let is_word = !c.is_whitespace();
        match current_is_word {
            Some(prev) if prev == is_word => current.push(c),
            Some(prev) => {
                segments.push(Segment {
                    text: std::mem::take(&mut current),
                    is_word: prev,
                });
                current.push(c);
                current_is_word = Some(is_word);
            }
            None => {
                current.push(c);
                current_is_word = Some(is_word);
            }
        }
    }
    if let Some(is_word) = current_is_word {
        segments.push(Segment {
            text: current,
            is_word,
        });
    }
    segments
}

/// The alphanumeric core of a word, ignoring surrounding punctuation
/// (`"of,"` -> `"of"`, `"(a)"` -> `"a"`).
pub fn core(word: &str) -> &str {
    word.trim_matches(|c: char| !c.is_alphanumeric())
}

/// Uppercase the first alphabetic character, leaving everything else untouched.
///
/// Unlike [`capitalize`], the remainder of the word keeps its original case, so
/// leading punctuation and already-cased tails are preserved.
pub fn capitalize_first_alpha(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    let mut done = false;
    for c in word.chars() {
        if !done && c.is_alphabetic() {
            result.extend(c.to_uppercase());
            done = true;
        } else {
            result.push(c);
        }
    }
    result
}

/// True if the word's core is a single alphabetic character (`"a"`, `"(I)"`).
pub fn is_single_letter(word: &str) -> bool {
    let c = core(word);
    c.chars().count() == 1 && c.chars().all(char::is_alphabetic)
}

/// True if the word contains any uppercase letter, i.e. it is already
/// capitalized in some way (`"In"`, `"iPhone"`, `"NASA"`).
pub fn has_uppercase(word: &str) -> bool {
    word.chars().any(char::is_uppercase)
}

/// True if the word's core is two-or-more letters and entirely uppercase
/// (`"NASA"`, `"LOUD"`) — i.e. a "shouting" all-caps word.
pub fn is_all_caps(word: &str) -> bool {
    let c = core(word);
    c.chars().count() >= 2
        && c.chars().any(char::is_alphabetic)
        && !c.chars().any(char::is_lowercase)
}

/// Built-in acronyms that should be fully capitalized when recognized.
///
/// Deliberately excludes short strings that collide with common English words
/// (`us`, `it`, `id`, `am`, …) to avoid uppercasing ordinary text. Extend or
/// replace at the CLI with `--acronyms`/`--no-acronyms`.
pub const DEFAULT_ACRONYMS: &[&str] = &[
    "ai", "api", "url", "uri", "html", "css", "js", "json", "xml", "yaml", "http", "https", "ftp",
    "ssh", "sql", "cpu", "gpu", "ram", "rom", "ssd", "hdd", "usb", "pdf", "png", "jpg", "jpeg",
    "gif", "svg", "csv", "tsv", "faq", "ceo", "cfo", "coo", "cto", "cio", "phd", "dna", "rna",
    "hiv", "usa", "uk", "eu", "un", "nasa", "fbi", "cia", "nsa", "atm", "gps", "sos", "diy",
    "nato", "utc", "gmt", "ascii", "utf", "wifi", "led", "lcd", "vip", "mvp", "suv", "io", "os",
    "ui", "ux", "pc", "tv", "gui", "cli", "sdk", "ide", "orm", "jwt", "tcp", "udp", "dns", "ssl",
    "tls", "hq", "qr", "kpi", "roi", "eta", "aka", "vpn", "vps", "cdn", "npm", "aws", "gcp", "iot",
    "ocr", "otp", "mfa", "sso", "saas", "paas", "iaas", "crud", "regex", "repl",
];

/// Minimum length for a consonant-only run to be treated as an acronym.
///
/// Two-letter clusters are excluded because many are ordinary abbreviations
/// that are *not* all-caps acronyms (`Mr`, `Dr`, `St`, `ft`, `vs`); genuine
/// two-letter acronyms (`js`, `tv`, `pc`, `io`) live in [`DEFAULT_ACRONYMS`]
/// instead. At three-plus letters, an all-consonant run is almost always an
/// initialism (`cnc`, `sql`, `xml`, `html`).
const MIN_HEURISTIC_ACRONYM_LEN: usize = 3;

/// Consonant-only strings that are *not* acronyms and must escape the
/// heuristic: interjections and a few abbreviations conventionally written
/// lower- or title-case. Without this, the shape rule would shout `hmm` into
/// `HMM` and `Mrs` into `MRS`. Entries are lowercase and matched against the
/// word's lowercased core.
const HEURISTIC_EXCEPTIONS: &[&str] = &[
    // interjections / onomatopoeia
    "hmm", "hmmm", "shh", "shhh", "pst", "psst", "brr", "brrr", "grr", "grrr", "tsk", "tsks",
    "pfft", "mmm", "mmhm", "zzz", "psh", // abbreviations conventionally not all-caps
    "mrs", "mr", "dr", "st", "nth",
];

/// True if `c` is a vowel for acronym-detection purposes.
///
/// `y` counts as a vowel here so that ordinary vowel-less English words
/// (`gym`, `why`, `dry`, `rhythm`) are not mistaken for acronyms.
fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Heuristic: does `word`'s core look like an acronym or an alphanumeric code,
/// without appearing in any list?
///
/// Two shapes qualify, and both require every *letter* to be a consonant so
/// ordinary words never match:
///
/// 1. **Consonant run** — three-or-more ASCII letters, no digits — how
///    initialisms like `cnc`, `sql`, and `html` read.
/// 2. **Letter-led code** — two-or-more consonant letters, led by a letter and
///    mixed with digits, e.g. part numbers like `bh1750`, `css3`, `ds18b20`.
///
/// Excluded, for robustness:
///
/// - anything with a vowel (`nasa`, `cat`, `chapter2`, `win10`) — pronounceable
///   acronyms are indistinguishable from words by shape and must come from the
///   acronym list; a word with a trailing number (`page5`) is likewise skipped;
/// - digit-led tokens — ordinals (`1st`), decades (`90s`), quantities (`3x`),
///   resolutions (`1080p`) — so those keep their conventional casing;
/// - single-letter-plus-digit tags (`v1`, `s3`, `p2`) — too ambiguous to shout;
///   list them explicitly if wanted;
/// - non-ASCII letters (`naïve`) and any internal punctuation (`x86_64`);
/// - bare two-letter consonant clusters (see [`MIN_HEURISTIC_ACRONYM_LEN`]);
/// - the [`HEURISTIC_EXCEPTIONS`] (`hmm`, `mrs`, `nth`, …).
fn looks_like_acronym(word: &str) -> bool {
    let c = core(word).to_lowercase();
    let mut letters = 0usize;
    let mut has_digit = false;
    for ch in c.chars() {
        if ch.is_ascii_digit() {
            has_digit = true;
            continue;
        }
        if !ch.is_ascii_alphabetic() {
            return false; // non-ASCII letter or internal punctuation
        }
        if is_vowel(ch) {
            return false;
        }
        letters += 1;
    }
    if letters == 0 {
        return false; // pure digits are not a code
    }
    if has_digit {
        // Letter-led consonant+digit code (`bh1750`). Two-plus letters, so
        // single-letter version tags (`v1`, `s3`) stay as written; a digit lead
        // (ordinal / decade / quantity) is likewise left alone.
        return letters >= 2 && c.starts_with(|ch: char| ch.is_ascii_alphabetic());
    }
    letters >= MIN_HEURISTIC_ACRONYM_LEN && !HEURISTIC_EXCEPTIONS.contains(&c.as_str())
}

/// A case-insensitive set of words to fully capitalize as acronyms.
pub struct AcronymSet {
    words: HashSet<String>,
    /// Whether to also treat vowel-less consonant runs as acronyms. Disabled by
    /// `--no-acronyms` so that flag turns off *all* acronym capitalization.
    use_heuristic: bool,
}

impl AcronymSet {
    /// Build a set from the built-in defaults (when `use_defaults`) plus any
    /// extra comma/space-separated entries.
    ///
    /// `use_defaults` also enables the consonant-run heuristic (see
    /// [`looks_like_acronym`]); it is only false for `--no-acronyms`, which
    /// disables acronym handling entirely.
    pub fn new(use_defaults: bool, extra: &[String]) -> Self {
        let mut words = HashSet::new();
        if use_defaults {
            words.extend(DEFAULT_ACRONYMS.iter().map(|s| s.to_string()));
        }
        for item in extra {
            for piece in item.split([',', ' ']) {
                let piece = piece.trim().to_lowercase();
                if !piece.is_empty() {
                    words.insert(piece);
                }
            }
        }
        Self {
            words,
            use_heuristic: use_defaults,
        }
    }

    /// True if the word's alphanumeric core is a recognized acronym — either a
    /// listed entry or, when the heuristic is enabled, a vowel-less consonant
    /// run (`cnc`, `sql`).
    pub fn matches(&self, word: &str) -> bool {
        let c = core(word).to_lowercase();
        if c.is_empty() {
            return false;
        }
        self.words.contains(&c) || (self.use_heuristic && looks_like_acronym(word))
    }
}

/// Capitalize the first letter of each sentence.
///
/// Intended to run over a whole block of text: a sentence starts at the very
/// beginning and after any `.`, `!`, or `?`. Newlines are neutral — they neither
/// start nor end a sentence — so a sentence wrapped across lines does not get
/// its continuation capitalized. Only lowercase letters are raised (existing
/// capitals are kept), and non-letters (digits, quotes, brackets) never trigger
/// capitalization.
pub fn capitalize_sentences(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut expect_capital = true;
    for c in line.chars() {
        if expect_capital && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            expect_capital = false;
        } else {
            out.push(c);
            if c.is_alphanumeric() {
                expect_capital = false;
            } else if matches!(c, '.' | '!' | '?') {
                expect_capital = true;
            }
        }
    }
    out
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

    #[test]
    fn segment_preserves_spacing() {
        let joined: String = segment("a  b c").into_iter().map(|s| s.text).collect();
        assert_eq!(joined, "a  b c");
        assert_eq!(segment("a  b").iter().filter(|s| s.is_word).count(), 2);
    }

    #[test]
    fn core_strips_edge_punctuation() {
        assert_eq!(core("of,"), "of");
        assert_eq!(core("(a)"), "a");
        assert_eq!(core("hello"), "hello");
    }

    #[test]
    fn capitalize_first_alpha_keeps_tail_and_punct() {
        assert_eq!(capitalize_first_alpha("(hello)"), "(Hello)");
        assert_eq!(capitalize_first_alpha("iPhone"), "IPhone");
        assert_eq!(capitalize_first_alpha("a"), "A");
    }

    #[test]
    fn single_letter_detection() {
        assert!(is_single_letter("a"));
        assert!(is_single_letter("(I)"));
        assert!(!is_single_letter("ab"));
        assert!(!is_single_letter("1"));
    }

    #[test]
    fn uppercase_predicates() {
        assert!(has_uppercase("In"));
        assert!(!has_uppercase("in"));
        assert!(is_all_caps("NASA"));
        assert!(!is_all_caps("Nasa"));
        assert!(!is_all_caps("I")); // single letter is not "shouting"
    }

    #[test]
    fn acronym_set_defaults_and_extra() {
        let set = AcronymSet::new(true, &[]);
        assert!(set.matches("api"));
        assert!(set.matches("(URL)")); // punctuation ignored via core()
        assert!(!set.matches("cat"));

        let custom = AcronymSet::new(true, &["foo, bar".to_string()]);
        assert!(custom.matches("foo"));
        assert!(custom.matches("bar"));

        let no_defaults = AcronymSet::new(false, &["baz".to_string()]);
        assert!(no_defaults.matches("baz"));
        assert!(!no_defaults.matches("api"));
    }

    #[test]
    fn consonant_run_heuristic_catches_unlisted_acronyms() {
        let set = AcronymSet::new(true, &[]);
        // Vowel-less runs of 3+ letters are acronyms even when unlisted.
        assert!(set.matches("cnc"));
        assert!(set.matches("dvr"));
        assert!(set.matches("sql"));
        assert!(set.matches("mgmt"));
        assert!(set.matches("(cnc)")); // punctuation ignored via core()
        assert!(set.matches("PWM")); // case-insensitive
    }

    #[test]
    fn heuristic_leaves_vowel_words_and_short_clusters_alone() {
        let set = AcronymSet::new(true, &[]);
        // Ordinary words with vowels are not acronyms by shape.
        assert!(!set.matches("cat"));
        assert!(!set.matches("world"));
        // `y` counts as a vowel, so these stay lowercase.
        assert!(!set.matches("gym"));
        assert!(!set.matches("why"));
        assert!(!set.matches("dry"));
        assert!(!set.matches("rhythm"));
        // Two-letter clusters are too ambiguous for the heuristic (Mr, Dr, vs).
        assert!(!set.matches("mr"));
        assert!(!set.matches("vs"));
        // A vowel in the letter portion means it is not a code.
        assert!(!set.matches("h2o"));
        // Single-letter-plus-digit tags are left alone.
        assert!(!set.matches("v1"));
        assert!(!set.matches("s3"));
        // Non-ASCII letters bail out rather than half-matching.
        assert!(!set.matches("naïve"));
    }

    #[test]
    fn heuristic_catches_letter_led_codes() {
        let set = AcronymSet::new(true, &[]);
        // Consonant letters mixed with digits are part-number-style codes.
        assert!(set.matches("bh1750"));
        assert!(set.matches("Bh1750")); // case-insensitive core
        assert!(set.matches("css3"));
        assert!(set.matches("mp3"));
        assert!(set.matches("ds18b20"));
    }

    #[test]
    fn heuristic_leaves_words_ordinals_and_decades_with_numbers_alone() {
        let set = AcronymSet::new(true, &[]);
        // Letter part has a vowel: an ordinary word with a number, not a code.
        assert!(!set.matches("chapter2"));
        assert!(!set.matches("page5"));
        assert!(!set.matches("win10"));
        assert!(!set.matches("covid19"));
        // Digit-led tokens keep conventional casing.
        assert!(!set.matches("1st"));
        assert!(!set.matches("2nd"));
        assert!(!set.matches("90s"));
        assert!(!set.matches("3x"));
        // Internal punctuation disqualifies it.
        assert!(!set.matches("x86_64"));
    }

    #[test]
    fn heuristic_exceptions_are_not_shouted() {
        let set = AcronymSet::new(true, &[]);
        // Interjections and conventional abbreviations stay as written.
        assert!(!set.matches("hmm"));
        assert!(!set.matches("shh"));
        assert!(!set.matches("brr"));
        assert!(!set.matches("zzz"));
        assert!(!set.matches("mrs"));
        assert!(!set.matches("nth"));
    }

    #[test]
    fn heuristic_still_covers_vowel_acronyms_via_list() {
        let set = AcronymSet::new(true, &[]);
        // Pronounceable acronyms can't be detected by shape; the list carries them.
        assert!(set.matches("nasa"));
        assert!(set.matches("api"));
        // A listed two-letter acronym still matches despite the length floor.
        assert!(set.matches("js"));
        // Newly listed common acronyms the shape rule can't catch.
        assert!(set.matches("hq"));
        assert!(set.matches("qr"));
        assert!(set.matches("kpi"));
        assert!(set.matches("roi"));
    }

    #[test]
    fn no_acronyms_disables_the_heuristic_too() {
        // `--no-acronyms` builds a set with defaults off; the consonant-run
        // heuristic must be off as well, not just the word list.
        let set = AcronymSet::new(false, &[]);
        assert!(!set.matches("cnc"));
        assert!(!set.matches("sql"));
    }

    #[test]
    fn sentence_capitalization() {
        assert_eq!(capitalize_sentences("hello. world"), "Hello. World");
        assert_eq!(capitalize_sentences("what? no! ok"), "What? No! Ok");
        // Existing capitals kept; digits do not trigger capitalization.
        assert_eq!(capitalize_sentences("3 cats. iOS wins"), "3 cats. IOS wins");
        assert_eq!(capitalize_sentences("(hi) there"), "(Hi) there");
    }

    #[test]
    fn sentence_capitalization_treats_newlines_as_neutral() {
        // Capitalize after a period across a newline, but not a wrapped line.
        assert_eq!(capitalize_sentences("one.\ntwo\nthree"), "One.\nTwo\nthree");
    }
}
