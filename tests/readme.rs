//! Keeps the README example honest: the example files it shows must match
//! `examples/basic`, and each `$ dotenv-diff ...` block must match the
//! binary's real output when run there.

use std::fs;
use std::path::Path;
use std::process::Command;

const README: &str = include_str!("../README.md");
const EXAMPLE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/basic");

/// Returns `(marker argument, fenced block body)` for each fenced block that
/// directly follows a `<!-- {marker}... -->` comment.
fn marked_blocks(marker: &str) -> Vec<(String, String)> {
    let prefix = format!("<!-- {marker}");
    let mut lines = README.lines();
    let mut blocks = Vec::new();

    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix(&prefix) else { continue };
        let arg = rest.trim_end_matches("-->").trim().to_string();

        let fence = lines.next().expect("marker must be followed by a code fence");
        assert!(fence.starts_with("```"), "expected code fence after `{line}`, got `{fence}`");

        let body: Vec<&str> = lines.by_ref().take_while(|l| !l.starts_with("```")).collect();
        blocks.push((arg, body.join("\n") + "\n"));
    }
    blocks
}

#[test]
fn readme_example_files_match_examples_dir() {
    let blocks = marked_blocks("example-file:");
    assert_eq!(blocks.len(), 2, "expected .env and .env.example blocks");

    for (name, shown) in blocks {
        let actual = fs::read_to_string(Path::new(EXAMPLE_DIR).join(&name)).unwrap();
        assert_eq!(shown, actual, "README block for {name} is out of date");
    }
}

#[test]
fn readme_example_output_matches_binary() {
    let blocks = marked_blocks("example-run");
    assert!(!blocks.is_empty(), "no example-run blocks found in README");

    for (_, block) in blocks {
        let (command, expected) = block.split_once('\n').unwrap();
        let mut words = command
            .strip_prefix("$ ")
            .expect("example-run block must start with `$ `")
            .split_whitespace();
        assert_eq!(words.next(), Some("dotenv-diff"));

        let out = Command::new(env!("CARGO_BIN_EXE_dotenv-diff"))
            .args(words)
            .current_dir(EXAMPLE_DIR)
            .output()
            .unwrap();

        assert_eq!(String::from_utf8(out.stdout).unwrap(), expected, "output of `{command}`");
        assert_eq!(out.status.code(), Some(1), "exit code of `{command}`");
    }
}
