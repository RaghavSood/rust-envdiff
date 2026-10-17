//! Argument parsing and the top-level `run` entry point.

use std::fs;
use std::io::Write;

use crate::diff::diff;
use crate::parse::{parse, Entry};
use crate::report::render;

pub const EXIT_OK: i32 = 0;
pub const EXIT_DIFF: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

pub const USAGE: &str = "\
Compare .env against .env.example and report missing or extra keys.

Usage: dotenv-diff [OPTIONS] [ENV] [EXAMPLE]

Arguments:
  [ENV]      Env file to check [default: .env]
  [EXAMPLE]  Template listing the expected keys [default: .env.example]

Options:
      --ignore-missing  Don't report or fail on keys missing from ENV
      --ignore-extra    Don't report or fail on keys not in EXAMPLE
  -q, --quiet           Print nothing; only set the exit code
  -h, --help            Print help
  -V, --version         Print version

Exit status: 0 if the files match, 1 if keys differ, 2 on error.
";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub env: String,
    pub example: String,
    pub ignore_missing: bool,
    pub ignore_extra: bool,
    pub quiet: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            env: ".env".into(),
            example: ".env.example".into(),
            ignore_missing: false,
            ignore_extra: false,
            quiet: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Run(Options),
    Help,
    Version,
}

/// Parses command-line arguments, excluding the program name.
pub fn parse_args<I, S>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut opts = Options::default();
    let mut positional = Vec::new();
    let mut only_positional = false;

    for arg in args.into_iter().map(Into::into) {
        if only_positional || !arg.starts_with('-') || arg == "-" {
            positional.push(arg);
            continue;
        }
        match arg.as_str() {
            "--" => only_positional = true,
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "-q" | "--quiet" => opts.quiet = true,
            "--ignore-missing" => opts.ignore_missing = true,
            "--ignore-extra" => opts.ignore_extra = true,
            _ => return Err(format!("unknown option `{arg}`")),
        }
    }

    let mut positional = positional.into_iter();
    if let Some(env) = positional.next() {
        opts.env = env;
    }
    if let Some(example) = positional.next() {
        opts.example = example;
    }
    if let Some(unexpected) = positional.next() {
        return Err(format!("unexpected argument `{unexpected}`"));
    }

    Ok(Command::Run(opts))
}

/// Runs the CLI and returns the process exit code.
pub fn run<I, S>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let opts = match parse_args(args) {
        Ok(Command::Run(opts)) => opts,
        Ok(Command::Help) => {
            let _ = stdout.write_all(USAGE.as_bytes());
            return EXIT_OK;
        }
        Ok(Command::Version) => {
            let _ = writeln!(stdout, "dotenv-diff {}", env!("CARGO_PKG_VERSION"));
            return EXIT_OK;
        }
        Err(msg) => {
            let _ = writeln!(stderr, "error: {msg}\n\nRun `dotenv-diff --help` for usage.");
            return EXIT_ERROR;
        }
    };

    let (env, example) = match (load(&opts.env), load(&opts.example)) {
        (Ok(env), Ok(example)) => (env, example),
        (Err(msg), _) | (_, Err(msg)) => {
            let _ = writeln!(stderr, "error: {msg}");
            return EXIT_ERROR;
        }
    };

    let mut d = diff(&env, &example);
    if opts.ignore_missing {
        d.missing.clear();
    }
    if opts.ignore_extra {
        d.extra.clear();
    }

    if !opts.quiet {
        let _ = stdout.write_all(render(&d, &opts.env, &opts.example).as_bytes());
    }

    if d.is_empty() {
        EXIT_OK
    } else {
        EXIT_DIFF
    }
}

fn load(path: &str) -> Result<Vec<Entry>, String> {
    let src = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    parse(&src).map_err(|e| format!("{path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_opts(args: &[&str]) -> Options {
        match parse_args(args.iter().copied()) {
            Ok(Command::Run(opts)) => opts,
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn defaults() {
        assert_eq!(run_opts(&[]), Options::default());
    }

    #[test]
    fn positional_paths() {
        let opts = run_opts(&["a.env"]);
        assert_eq!((opts.env.as_str(), opts.example.as_str()), ("a.env", ".env.example"));

        let opts = run_opts(&["a.env", "b.env"]);
        assert_eq!((opts.env.as_str(), opts.example.as_str()), ("a.env", "b.env"));
    }

    #[test]
    fn flags_in_any_position() {
        let opts = run_opts(&["--ignore-extra", "a", "-q", "b", "--ignore-missing"]);
        assert_eq!(opts.env, "a");
        assert_eq!(opts.example, "b");
        assert!(opts.quiet);
        assert!(opts.ignore_missing && opts.ignore_extra);
    }

    #[test]
    fn double_dash_ends_options() {
        let opts = run_opts(&["--", "-weird", "--quiet"]);
        assert_eq!(opts.env, "-weird");
        assert_eq!(opts.example, "--quiet");
        assert!(!opts.quiet);
    }

    #[test]
    fn help_and_version() {
        assert_eq!(parse_args(["-h"]), Ok(Command::Help));
        assert_eq!(parse_args(["a", "--help"]), Ok(Command::Help));
        assert_eq!(parse_args(["-V"]), Ok(Command::Version));
        assert_eq!(parse_args(["--version"]), Ok(Command::Version));
    }

    #[test]
    fn rejects_unknown_option() {
        assert_eq!(parse_args(["--nope"]), Err("unknown option `--nope`".into()));
    }

    #[test]
    fn rejects_too_many_paths() {
        assert_eq!(parse_args(["a", "b", "c"]), Err("unexpected argument `c`".into()));
    }
}
