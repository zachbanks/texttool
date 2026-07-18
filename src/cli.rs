//! Command-line surface: builds a colored `clap` command tree from the
//! [`Registry`] and dispatches a parsed invocation to the right transform.
//!
//! The command tree is generated *from data* — one subcommand per registered
//! transform — so the CLI, its help text, and its colors all stay in sync with
//! the registry automatically.

use crate::io::{add_io_args, read_input, write_output};
use crate::registry::Registry;
use clap::builder::styling::{AnsiColor, Styles};
use clap::{ArgMatches, Command};

/// Long description shown on `tt --help`.
const LONG_ABOUT: &str = "\
tt is a unified, extensible toolkit for manipulating text.

Each operation is exposed as a subcommand. Every subcommand reads from the file
operands you pass, or from standard input when you pass none, and writes to
standard output unless you redirect it with --output.

Examples:
  tt titlecase notes.txt
  echo 'the lord of the rings' | tt titlecase
  tt clean --ascii messy.md -o clean.md
  cat a.txt b.txt | tt upper";

/// Colors used throughout help and error output.
///
/// Centralised here so the whole CLI shares one palette.
fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().bold())
        .valid(AnsiColor::Green.on_default().bold())
        .invalid(AnsiColor::Yellow.on_default().bold())
}

/// Build the full `clap` command tree from the registry.
///
/// One subcommand is generated per registered transform, wired with that
/// transform's own flags plus the shared input/output arguments.
pub fn build_command(registry: &Registry) -> Command {
    let mut cmd = Command::new("tt")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Unified, extensible toolkit for text manipulation")
        .long_about(LONG_ABOUT)
        .styles(styles())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .propagate_version(true);

    for transform in registry.all() {
        let mut sub = Command::new(transform.name())
            .about(transform.about())
            .visible_aliases(transform.aliases().to_vec());
        if let Some(long) = transform.long_about() {
            sub = sub.long_about(long);
        }
        sub = transform.augment(sub);
        sub = add_io_args(sub);
        cmd = cmd.subcommand(sub);
    }

    cmd
}

/// Run a fully-parsed invocation: read input, apply the selected transform,
/// write output. Returns a human-readable error message on failure.
pub fn dispatch(registry: &Registry, matches: &ArgMatches) -> Result<(), String> {
    let (name, sub_matches) = matches
        .subcommand()
        .ok_or_else(|| "no subcommand provided".to_string())?;

    let transform = registry
        .get(name)
        .ok_or_else(|| format!("unknown transform `{name}`"))?;

    let input = read_input(sub_matches)?;
    let output = transform.apply(&input, sub_matches)?;
    write_output(sub_matches, &output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::default_registry;

    #[test]
    fn command_tree_builds_and_validates() {
        // `debug_assert` inside clap catches malformed command definitions.
        build_command(&default_registry()).debug_assert();
    }

    #[test]
    fn every_transform_has_a_subcommand() {
        let registry = default_registry();
        let cmd = build_command(&registry);
        for transform in registry.all() {
            assert!(
                cmd.get_subcommands()
                    .any(|s| s.get_name() == transform.name()),
                "missing subcommand for `{}`",
                transform.name()
            );
        }
    }

    #[test]
    fn subcommand_about_lines_include_examples() {
        let cmd = build_command(&default_registry());
        for subcommand in cmd.get_subcommands() {
            let about = format!("{}", subcommand.get_about().expect("subcommand about"));
            assert!(
                about.contains("[e.g.") && about.ends_with(']'),
                "help summary for `{}` should end with an example: {about}",
                subcommand.get_name()
            );
        }
    }
}
