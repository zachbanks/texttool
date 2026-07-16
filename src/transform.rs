//! The core extension point of `texttool`.
//!
//! Every text operation the CLI exposes is a type that implements
//! [`Transform`]. This is a small application of the *strategy pattern*: the
//! CLI machinery (argument parsing, I/O, dispatch, help generation) is written
//! once against the trait, and each concrete operation plugs in without the
//! surrounding code having to know anything about it.
//!
//! Adding a new operation is deliberately mechanical:
//!
//! 1. Create a type and `impl Transform for MyThing`.
//! 2. Register it in [`crate::registry::default_registry`].
//!
//! No changes to argument parsing or `main` are required — the new operation
//! automatically gains a colored, documented subcommand.

use clap::{ArgMatches, Command};

/// A single, self-contained text transformation.
///
/// Implementors receive the *entire* input as a `&str` and return the
/// transformed output as an owned `String`. Working on the whole buffer (rather
/// than line-by-line) keeps the trait simple and lets each transform decide for
/// itself how to treat line boundaries.
pub trait Transform {
    /// Canonical machine name, used as the subcommand (e.g. `clean`).
    ///
    /// Must be unique within a [`Registry`](crate::registry::Registry) and
    /// should be a short, lowercase, single word.
    fn name(&self) -> &'static str;

    /// One-line summary shown in the top-level `--help` listing.
    fn about(&self) -> &'static str;

    /// Optional longer description shown on the subcommand's own `--help`.
    fn long_about(&self) -> Option<&'static str> {
        None
    }

    /// Optional additional names the subcommand also answers to.
    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    /// Hook to add transform-specific arguments/flags to the generated
    /// subcommand. The default adds nothing.
    ///
    /// The common input/output arguments (file operands, `--output`) are added
    /// separately by the CLI builder, so implementors only declare flags unique
    /// to their operation here.
    fn augment(&self, cmd: Command) -> Command {
        cmd
    }

    /// Apply the transformation.
    ///
    /// `args` are the parsed matches for this transform's subcommand, giving
    /// access to any flags declared in [`augment`](Transform::augment).
    /// Returning `Err` reports a human-readable message and exits non-zero.
    fn apply(&self, input: &str, args: &ArgMatches) -> Result<String, String>;
}
