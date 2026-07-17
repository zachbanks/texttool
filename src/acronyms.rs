//! User-configurable acronym handling shared by `clean` and `titlecase`.
//!
//! Acronyms are recognized from three layered sources, in addition to the
//! built-in default list:
//!
//! 1. A persistent config file (so you don't retype `--acronyms` every time):
//!    `$TEXTTOOL_ACRONYMS_FILE`, else `$XDG_CONFIG_HOME/texttool/acronyms.txt`,
//!    else `~/.config/texttool/acronyms.txt`.
//! 2. A per-invocation `--acronyms-file <PATH>`.
//! 3. Inline `--acronyms a,b,c`.
//!
//! `--no-acronyms` disables all acronym capitalization (built-ins included).

use crate::casing::AcronymSet;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::path::{Path, PathBuf};
use std::{env, fs};

/// Attach the shared `--acronyms` / `--acronyms-file` / `--no-acronyms` flags.
pub fn add_acronym_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("acronyms")
            .long("acronyms")
            .value_name("LIST")
            .help("Extra acronyms to capitalize [e.g. --acronyms tui,repl: \"tui\" -> \"TUI\"]")
            .value_delimiter(',')
            .action(ArgAction::Append),
    )
    .arg(
        Arg::new("acronyms-file")
            .long("acronyms-file")
            .value_name("PATH")
            .help("Read extra acronyms from a file (one per line; # comments allowed)"),
    )
    .arg(
        Arg::new("no-acronyms")
            .long("no-acronyms")
            .help("Disable acronym capitalization [e.g. \"api\" stays lowercase]")
            .action(ArgAction::SetTrue),
    )
}

/// Resolve the default persistent config path, honoring the environment.
fn default_config_path() -> Option<PathBuf> {
    if let Some(path) = env::var_os("TEXTTOOL_ACRONYMS_FILE") {
        return Some(PathBuf::from(path));
    }
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("texttool").join("acronyms.txt"))
}

/// Read acronyms from a file, ignoring `#` comments and blank entries.
///
/// A missing or unreadable file yields no entries (it is optional config).
fn load_file(path: &Path) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    contents
        .lines()
        .map(|line| line.split('#').next().unwrap_or("")) // strip comments
        .flat_map(|line| line.split([',', ' ', '\t']))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

/// Build the acronym set for this invocation from all configured sources.
pub fn build_acronym_set(args: &ArgMatches) -> AcronymSet {
    if args.get_flag("no-acronyms") {
        return AcronymSet::new(false, &[]);
    }

    let mut extra: Vec<String> = Vec::new();
    if let Some(path) = default_config_path() {
        extra.extend(load_file(&path));
    }
    if let Some(path) = args.get_one::<String>("acronyms-file") {
        extra.extend(load_file(Path::new(path)));
    }
    if let Some(values) = args.get_many::<String>("acronyms") {
        extra.extend(values.cloned());
    }
    AcronymSet::new(true, &extra)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_file_parses_comments_and_separators() {
        let mut file = tempfile();
        writeln!(file.as_file_mut(), "# my acronyms").unwrap();
        writeln!(file.as_file_mut(), "tui, repl").unwrap();
        writeln!(file.as_file_mut(), "gpu  # inline comment").unwrap();
        let entries = load_file(file.path());
        assert!(entries.contains(&"tui".to_string()));
        assert!(entries.contains(&"repl".to_string()));
        assert!(entries.contains(&"gpu".to_string()));
        assert!(!entries.iter().any(|e| e.contains('#')));
    }

    #[test]
    fn missing_file_is_empty() {
        assert!(load_file(Path::new("/no/such/texttool/acronyms.txt")).is_empty());
    }

    /// Minimal temp-file helper (no external crates): a uniquely named file in
    /// the OS temp dir that is removed on drop.
    struct TempFile {
        path: PathBuf,
        file: fs::File,
    }
    impl TempFile {
        fn as_file_mut(&mut self) -> &mut fs::File {
            &mut self.file
        }
        fn path(&self) -> &Path {
            &self.path
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }
    fn tempfile() -> TempFile {
        let mut path = env::temp_dir();
        let unique = format!(
            "texttool-acronyms-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        path.push(unique);
        let file = fs::File::create(&path).unwrap();
        TempFile { path, file }
    }
}
