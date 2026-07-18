//! User-configurable extraction categories for the `extract` transform.
//!
//! A category is a named regular expression. The set is the built-in defaults,
//! overridden and extended by a TOML config file:
//!
//! 1. `$TEXTTOOL_PATTERNS_FILE`, else
//! 2. `$XDG_CONFIG_HOME/texttool/patterns.toml`, else
//! 3. `~/.config/texttool/patterns.toml`
//!
//! plus a per-invocation `--patterns-file <PATH>`.
//!
//! ```toml
//! # patterns.toml — array of tables, evaluated in order.
//! [[category]]
//! name = "Phone Numbers"
//! regex = '''(?:\+?1[\s.-]?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}'''
//!
//! [[category]]
//! name = "Ticket IDs"
//! regex = '''\bJIRA-\d+\b'''
//!
//! # Disable a built-in by name:
//! [[category]]
//! name = "ZIP Codes"
//! enabled = false
//! ```

use clap::{Arg, ArgMatches, Command};
use std::path::{Path, PathBuf};
use std::{env, fs};

/// A single extraction category: a display name and a regex pattern.
#[derive(Clone)]
pub struct Category {
    pub name: String,
    pub regex: String,
}

/// The built-in categories, in output order. Patterns use `regex`-crate syntax
/// (no lookaround/backreferences).
pub fn default_patterns() -> Vec<Category> {
    let defs: &[(&str, &str)] = &[
        (
            "Phone Numbers",
            r"(?:\+?1[\s.-]?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}",
        ),
        (
            "Emails",
            r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
        ),
        ("URLs", r"(?:https?://|www\.)[^\s<>()]*[^\s<>().,!?;:]"),
        (
            "Addresses",
            r"(?i)\b\d{1,6}\s+(?:[A-Za-z0-9.'-]+\s+){0,4}(?:street|st|avenue|ave|boulevard|blvd|road|rd|lane|ln|drive|dr|court|ct|way|terrace|ter|place|pl|circle|cir|highway|hwy|parkway|pkwy)\b\.?",
        ),
        (
            "Dates",
            r"(?i)\b(?:\d{4}[/.-]\d{1,2}[/.-]\d{1,2}|\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}|(?:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\.?\s+\d{1,2}(?:st|nd|rd|th)?,?\s+\d{2,4}|\d{1,2}(?:st|nd|rd|th)?\s+(?:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\.?,?\s+\d{2,4})\b",
        ),
        (
            "Times",
            r"(?i)\b\d{1,2}:\d{2}(?::\d{2})?\s?(?:[ap]\.?m\.?)?\b",
        ),
        ("SSNs", r"\b\d{3}-\d{2}-\d{4}\b"),
        ("Credit Cards", r"\b(?:\d{4}[\s-]?){3}\d{1,4}\b"),
        ("IP Addresses", r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
    ];
    defs.iter()
        .map(|(name, regex)| Category {
            name: (*name).to_string(),
            regex: (*regex).to_string(),
        })
        .collect()
}

/// Attach the shared `--patterns-file` argument.
pub fn add_pattern_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("patterns-file")
            .long("patterns-file")
            .value_name("PATH")
            .help("Load extra/override categories from a TOML file"),
    )
}

/// Resolve the default config path, honoring the environment.
fn default_config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("TEXTTOOL_PATTERNS_FILE") {
        return Some(PathBuf::from(path));
    }
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("texttool").join("patterns.toml"))
}

/// One category entry parsed from config: a rename/override or a disable.
struct ConfigEntry {
    name: String,
    regex: Option<String>,
    enabled: bool,
}

/// Parse `[[category]]` tables from a TOML string.
fn parse_config(contents: &str) -> Result<Vec<ConfigEntry>, String> {
    let table: toml::Table = contents
        .parse()
        .map_err(|e| format!("invalid patterns TOML: {e}"))?;
    let Some(array) = table.get("category") else {
        return Ok(Vec::new());
    };
    let array = array
        .as_array()
        .ok_or_else(|| "`category` must be an array of tables ([[category]])".to_string())?;

    let mut entries = Vec::new();
    for item in array {
        let item = item
            .as_table()
            .ok_or_else(|| "each `[[category]]` entry must be a table".to_string())?;
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "each `[[category]]` needs a string `name`".to_string())?
            .to_string();
        let regex = item
            .get("regex")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let enabled = item
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        entries.push(ConfigEntry {
            name,
            regex,
            enabled,
        });
    }
    Ok(entries)
}

/// Merge config entries into the ordered category list: override built-ins by
/// name, disable those set `enabled = false`, and append brand-new categories.
fn merge(mut categories: Vec<Category>, entries: Vec<ConfigEntry>) -> Vec<Category> {
    for entry in entries {
        let existing = categories
            .iter_mut()
            .find(|c| c.name.eq_ignore_ascii_case(&entry.name));
        match existing {
            Some(category) => {
                if let Some(regex) = &entry.regex {
                    category.regex = regex.clone();
                }
                if !entry.enabled {
                    category.name.clear(); // mark for removal
                }
            }
            None if entry.enabled => {
                if let Some(regex) = entry.regex {
                    categories.push(Category {
                        name: entry.name,
                        regex,
                    });
                }
            }
            None => {}
        }
    }
    categories.retain(|c| !c.name.is_empty());
    categories
}

/// Build the full category list for this invocation (built-ins + config).
pub fn load_categories(args: &ArgMatches) -> Result<Vec<Category>, String> {
    let mut categories = default_patterns();

    if let Some(path) = default_config_path()
        && let Ok(contents) = fs::read_to_string(&path)
    {
        categories = merge(categories, parse_config(&contents)?);
    }
    if let Some(path) = args.get_one::<String>("patterns-file") {
        let contents = fs::read_to_string(Path::new(path))
            .map_err(|e| format!("failed to read `{path}`: {e}"))?;
        categories = merge(categories, parse_config(&contents)?);
    }
    Ok(categories)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_present_and_ordered() {
        let cats = default_patterns();
        assert_eq!(cats.first().unwrap().name, "Phone Numbers");
        assert!(cats.iter().any(|c| c.name == "Emails"));
    }

    #[test]
    fn config_overrides_and_extends() {
        let toml = r#"
[[category]]
name = "Emails"
regex = "OVERRIDDEN"

[[category]]
name = "Ticket IDs"
regex = "JIRA-\\d+"

[[category]]
name = "SSNs"
enabled = false
"#;
        let merged = merge(default_patterns(), parse_config(toml).unwrap());
        let email = merged.iter().find(|c| c.name == "Emails").unwrap();
        assert_eq!(email.regex, "OVERRIDDEN");
        assert!(merged.iter().any(|c| c.name == "Ticket IDs"));
        assert!(!merged.iter().any(|c| c.name == "SSNs"));
    }

    #[test]
    fn invalid_toml_errors() {
        assert!(parse_config("not = = valid").is_err());
    }
}
