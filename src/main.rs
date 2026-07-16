//! `texttool` — a unified, extensible command-line toolkit for text
//! manipulation.
//!
//! The binary is deliberately thin: it builds the [`Registry`] of operations,
//! turns it into a colored `clap` command tree, parses arguments, and hands off
//! to [`cli::dispatch`]. All real behaviour lives in the individual
//! [`Transform`](transform::Transform) implementations under [`transforms`].

mod cli;
mod io;
mod registry;
mod transform;
mod transforms;

use std::process::ExitCode;

fn main() -> ExitCode {
    let registry = registry::default_registry();
    let command = cli::build_command(&registry);
    let matches = command.get_matches();

    match cli::dispatch(&registry, &matches) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("texttool: error: {message}");
            ExitCode::FAILURE
        }
    }
}
