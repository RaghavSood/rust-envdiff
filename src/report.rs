//! Human-readable rendering of a [`Diff`].

use std::fmt::Write;

use crate::diff::Diff;
use crate::parse::Entry;

pub fn render(diff: &Diff, env_path: &str, example_path: &str) -> String {
    let Diff { missing, extra } = diff;

    if diff.is_empty() {
        return format!("OK: {env_path} matches {example_path}\n");
    }

    let mut out = String::new();
    if !missing.is_empty() {
        let _ = writeln!(out, "Missing from {env_path} ({}):", missing.len());
        write_entries(&mut out, '-', missing, example_path);
    }
    if !extra.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(out, "Extra in {env_path} ({}):", extra.len());
        write_entries(&mut out, '+', extra, env_path);
    }
    out
}

fn write_entries(out: &mut String, marker: char, entries: &[Entry], path: &str) {
    let width = entries.iter().map(|e| e.key.len()).max().unwrap_or(0);
    for e in entries {
        let _ = writeln!(out, "  {marker} {:<width$}  ({path}:{})", e.key, e.line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: &str, line: usize) -> Entry {
        Entry { key: key.into(), line }
    }

    #[test]
    fn in_sync() {
        let out = render(&Diff::default(), ".env", ".env.example");
        assert_eq!(out, "OK: .env matches .env.example\n");
    }

    #[test]
    fn missing_and_extra_with_aligned_keys() {
        let diff = Diff {
            missing: vec![entry("DATABASE_URL", 2), entry("PORT", 5)],
            extra: vec![entry("DEBUG", 4)],
        };
        let out = render(&diff, ".env", ".env.example");
        assert_eq!(
            out,
            "Missing from .env (2):\n\
             \x20 - DATABASE_URL  (.env.example:2)\n\
             \x20 - PORT          (.env.example:5)\n\
             \n\
             Extra in .env (1):\n\
             \x20 + DEBUG  (.env:4)\n"
        );
    }

    #[test]
    fn only_extra() {
        let diff = Diff { missing: vec![], extra: vec![entry("X", 1)] };
        let out = render(&diff, "a", "b");
        assert_eq!(out, "Extra in a (1):\n  + X  (a:1)\n");
    }
}
