//! Shared input/output plumbing for every subcommand.
//!
//! All transforms read the same way (one or more file operands, or stdin when
//! none are given) and write the same way (stdout, or `--output <FILE>`). That
//! shared behaviour lives here so individual transforms never touch the
//! filesystem or streams directly.

use clap::{Arg, ArgAction, Command};
use std::fs;
use std::io::{self, Read, Write};

/// Name of the positional file operands argument.
pub const FILES_ARG: &str = "files";
/// Name of the `--output` argument.
pub const OUTPUT_ARG: &str = "output";

/// Attach the common I/O arguments to a subcommand.
///
/// Added to every generated subcommand so that file/stdin input and
/// stdout/file output work identically no matter which transform runs.
pub fn add_io_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new(FILES_ARG)
            .help("Input files to read; reads standard input when omitted")
            .value_name("FILE")
            .num_args(0..)
            .action(ArgAction::Append),
    )
    .arg(
        Arg::new(OUTPUT_ARG)
            .short('o')
            .long("output")
            .help("Write result to this file instead of standard output")
            .value_name("FILE"),
    )
}

/// Read the transform's input from the file operands, or from stdin if none.
///
/// Multiple files are concatenated in the order given.
pub fn read_input(args: &clap::ArgMatches) -> Result<String, String> {
    let files: Vec<&String> = args
        .get_many::<String>(FILES_ARG)
        .map(|values| values.collect())
        .unwrap_or_default();

    if files.is_empty() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| format!("failed to read standard input: {e}"))?;
        return Ok(buffer);
    }

    let mut buffer = String::new();
    for path in files {
        let contents =
            fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
        buffer.push_str(&contents);
    }
    Ok(buffer)
}

/// Write the transform's output to `--output <FILE>`, or to stdout.
pub fn write_output(args: &clap::ArgMatches, output: &str) -> Result<(), String> {
    match args.get_one::<String>(OUTPUT_ARG) {
        Some(path) => fs::write(path, output).map_err(|e| format!("failed to write `{path}`: {e}")),
        None => io::stdout()
            .write_all(output.as_bytes())
            .map_err(|e| format!("failed to write standard output: {e}")),
    }
}
