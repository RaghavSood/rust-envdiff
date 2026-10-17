//! Key-set comparison between an env file and its example template.

use std::collections::HashSet;

use crate::parse::Entry;

/// Result of comparing an env file against an example file.
///
/// Each list is ordered by first appearance in the file the key comes from,
/// and each key appears at most once.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Diff {
    /// Keys in the example file that the env file does not define.
    pub missing: Vec<Entry>,
    /// Keys in the env file that the example file does not define.
    pub extra: Vec<Entry>,
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self.missing.is_empty() && self.extra.is_empty()
    }
}

pub fn diff(env: &[Entry], example: &[Entry]) -> Diff {
    Diff {
        missing: difference(example, env),
        extra: difference(env, example),
    }
}

/// Entries of `a` whose key is absent from `b`, first occurrence only.
fn difference(a: &[Entry], b: &[Entry]) -> Vec<Entry> {
    let in_b: HashSet<&str> = b.iter().map(|e| e.key.as_str()).collect();
    let mut seen = HashSet::new();
    a.iter()
        .filter(|e| !in_b.contains(e.key.as_str()) && seen.insert(e.key.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(keys: &[&str]) -> Vec<Entry> {
        keys.iter()
            .enumerate()
            .map(|(i, k)| Entry { key: k.to_string(), line: i + 1 })
            .collect()
    }

    fn keys(entries: &[Entry]) -> Vec<&str> {
        entries.iter().map(|e| e.key.as_str()).collect()
    }

    #[test]
    fn identical_sets_are_empty() {
        let d = diff(&entries(&["A", "B"]), &entries(&["B", "A"]));
        assert!(d.is_empty());
    }

    #[test]
    fn reports_missing_keys() {
        let d = diff(&entries(&["A"]), &entries(&["A", "B", "C"]));
        assert_eq!(keys(&d.missing), ["B", "C"]);
        assert!(d.extra.is_empty());
    }

    #[test]
    fn reports_extra_keys() {
        let d = diff(&entries(&["A", "X", "Y"]), &entries(&["A"]));
        assert!(d.missing.is_empty());
        assert_eq!(keys(&d.extra), ["X", "Y"]);
    }

    #[test]
    fn reports_both_directions() {
        let d = diff(&entries(&["A", "X"]), &entries(&["A", "B"]));
        assert_eq!(keys(&d.missing), ["B"]);
        assert_eq!(keys(&d.extra), ["X"]);
        assert!(!d.is_empty());
    }

    #[test]
    fn line_numbers_come_from_source_file() {
        let d = diff(&entries(&["A", "X"]), &entries(&["B", "C", "A"]));
        assert_eq!(d.missing, entries(&["B", "C"]));
        assert_eq!(d.extra, [Entry { key: "X".into(), line: 2 }]);
    }

    #[test]
    fn duplicates_are_reported_once_at_first_occurrence() {
        let d = diff(&entries(&["X", "X"]), &entries(&["B", "B"]));
        assert_eq!(d.missing, [Entry { key: "B".into(), line: 1 }]);
        assert_eq!(d.extra, [Entry { key: "X".into(), line: 1 }]);
    }

    #[test]
    fn keys_are_case_sensitive() {
        let d = diff(&entries(&["api_key"]), &entries(&["API_KEY"]));
        assert_eq!(keys(&d.missing), ["API_KEY"]);
        assert_eq!(keys(&d.extra), ["api_key"]);
    }

    #[test]
    fn empty_inputs() {
        assert!(diff(&[], &[]).is_empty());
        assert_eq!(keys(&diff(&[], &entries(&["A"])).missing), ["A"]);
        assert_eq!(keys(&diff(&entries(&["A"]), &[]).extra), ["A"]);
    }
}
