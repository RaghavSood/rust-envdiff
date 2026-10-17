//! End-to-end tests that run the compiled binary against real files.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A scratch directory removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("dotenv-diff-{}-{n}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }

    fn write(&self, name: &str, contents: &str) -> &Self {
        fs::write(self.0.join(name), contents).unwrap();
        self
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dotenv-diff"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).unwrap()
}

#[test]
fn matching_files_exit_zero() {
    let dir = TempDir::new();
    dir.write(".env", "A=1\nB=2\n").write(".env.example", "B=\nA=\n");

    let out = dir.run(&[]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "OK: .env matches .env.example\n");
}

#[test]
fn differences_exit_one() {
    let dir = TempDir::new();
    dir.write(".env", "A=1\nEXTRA=1\n").write(".env.example", "A=\nNEEDED=\n");

    let out = dir.run(&[]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        stdout(&out),
        "Missing from .env (1):\n  - NEEDED  (.env.example:2)\n\n\
         Extra in .env (1):\n  + EXTRA  (.env:2)\n"
    );
}

#[test]
fn custom_paths() {
    let dir = TempDir::new();
    dir.write("prod.env", "A=1\n").write("template", "A=\nB=\n");

    let out = dir.run(&["prod.env", "template"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "Missing from prod.env (1):\n  - B  (template:2)\n");
}

#[test]
fn quiet_prints_nothing() {
    let dir = TempDir::new();
    dir.write(".env", "A=1\n").write(".env.example", "B=\n");

    let out = dir.run(&["-q"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn ignore_extra() {
    let dir = TempDir::new();
    dir.write(".env", "A=1\nLOCAL=1\n").write(".env.example", "A=\n");

    let out = dir.run(&["--ignore-extra"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "OK: .env matches .env.example\n");
}

#[test]
fn ignore_missing() {
    let dir = TempDir::new();
    dir.write(".env", "A=1\nLOCAL=1\n").write(".env.example", "A=\nB=\n");

    let out = dir.run(&["--ignore-missing"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "Extra in .env (1):\n  + LOCAL  (.env:2)\n");
}

#[test]
fn missing_file_exits_two() {
    let dir = TempDir::new();
    dir.write(".env.example", "A=\n");

    let out = dir.run(&[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(stderr(&out).starts_with("error: cannot read .env:"), "{}", stderr(&out));
}

#[test]
fn parse_error_exits_two() {
    let dir = TempDir::new();
    dir.write(".env", "A=1\noops\n").write(".env.example", "A=\n");

    let out = dir.run(&[]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stderr(&out), "error: .env: line 2: expected KEY=VALUE, found `oops`\n");
}

#[test]
fn unknown_option_exits_two() {
    let out = TempDir::new().run(&["--bogus"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("unknown option `--bogus`"));
}

#[test]
fn help_and_version() {
    let dir = TempDir::new();

    let out = dir.run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("Usage: dotenv-diff"));

    let out = dir.run(&["--version"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), format!("dotenv-diff {}\n", env!("CARGO_PKG_VERSION")));
}
