//! Minimal dotenv parser that extracts keys (values are ignored).
//!
//! Supported syntax:
//! - `KEY=value`, with optional whitespace around `=`
//! - `export KEY=value`
//! - blank lines and `#` comments
//! - single-, double- and backtick-quoted values, including quoted values
//!   that span multiple lines

use std::fmt;

/// A key found in a dotenv file, with the 1-based line it was defined on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parses dotenv source text and returns its entries in file order.
///
/// Duplicate keys are kept; callers decide how to treat them.
pub fn parse(src: &str) -> Result<Vec<Entry>, ParseError> {
    let mut entries = Vec::new();
    let mut lines = src.lines().enumerate().map(|(i, l)| (i + 1, l));

    while let Some((line_no, raw)) = lines.next() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = strip_export(line);
        let Some((key, value)) = line.split_once('=') else {
            return Err(ParseError {
                line: line_no,
                message: format!("expected KEY=VALUE, found `{}`", raw.trim()),
            });
        };

        let key = key.trim();
        if !is_valid_key(key) {
            return Err(ParseError {
                line: line_no,
                message: format!("invalid key `{key}`"),
            });
        }

        // A quoted value that is not closed on this line continues onto the
        // following lines; skip them so their contents aren't read as keys.
        let value = value.trim_start();
        if let Some(quote) = value.chars().next().filter(|c| matches!(c, '"' | '\'' | '`')) {
            if find_closing_quote(&value[1..], quote).is_none() {
                let closed = lines
                    .by_ref()
                    .any(|(_, l)| find_closing_quote(l, quote).is_some());
                if !closed {
                    return Err(ParseError {
                        line: line_no,
                        message: format!("unterminated {quote} quote for key `{key}`"),
                    });
                }
            }
        }

        entries.push(Entry {
            key: key.to_string(),
            line: line_no,
        });
    }

    Ok(entries)
}

fn strip_export(line: &str) -> &str {
    match line.strip_prefix("export") {
        Some(rest) if rest.starts_with(char::is_whitespace) => rest.trim_start(),
        _ => line,
    }
}

/// Keys must look like shell identifiers, plus `.` and `-` which many
/// dotenv implementations accept.
fn is_valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// Returns the byte index of the first unescaped `quote` in `s`.
/// Backslash escapes only apply inside double quotes.
fn find_closing_quote(s: &str, quote: char) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' && quote == '"' {
            escaped = true;
        } else if c == quote {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(src: &str) -> Vec<String> {
        parse(src).unwrap().into_iter().map(|e| e.key).collect()
    }

    #[test]
    fn parses_simple_pairs() {
        assert_eq!(keys("A=1\nB=2\n"), ["A", "B"]);
    }

    #[test]
    fn records_line_numbers() {
        let entries = parse("\n# comment\nA=1\n\nB=2").unwrap();
        assert_eq!(
            entries,
            [
                Entry { key: "A".into(), line: 3 },
                Entry { key: "B".into(), line: 5 },
            ]
        );
    }

    #[test]
    fn skips_blank_lines_and_comments() {
        assert_eq!(keys("# header\n\n   \n  # indented\nA=1"), ["A"]);
    }

    #[test]
    fn allows_empty_values() {
        assert_eq!(keys("A=\nB= \n"), ["A", "B"]);
    }

    #[test]
    fn trims_whitespace_around_key() {
        assert_eq!(keys("  A  = 1"), ["A"]);
    }

    #[test]
    fn strips_export_prefix() {
        assert_eq!(keys("export A=1\nexport\tB=2"), ["A", "B"]);
    }

    #[test]
    fn export_prefix_requires_whitespace() {
        assert_eq!(keys("exporter=1"), ["exporter"]);
    }

    #[test]
    fn value_may_contain_equals_and_hash() {
        assert_eq!(keys("URL=postgres://u:p@h/db?a=b#frag"), ["URL"]);
    }

    #[test]
    fn accepts_dots_and_dashes_in_keys() {
        assert_eq!(keys("app.name=x\nmy-key=y"), ["app.name", "my-key"]);
    }

    #[test]
    fn keeps_duplicates() {
        assert_eq!(keys("A=1\nA=2"), ["A", "A"]);
    }

    #[test]
    fn single_line_quoted_values() {
        assert_eq!(keys("A=\"x y\"\nB='z'\nC=`w`"), ["A", "B", "C"]);
    }

    #[test]
    fn multiline_double_quoted_value() {
        let src = "KEY=\"-----BEGIN-----\nNOT_A_KEY=1\n-----END-----\"\nNEXT=1";
        assert_eq!(keys(src), ["KEY", "NEXT"]);
    }

    #[test]
    fn escaped_quote_does_not_close_double_quoted_value() {
        let src = "A=\"say \\\"hi\\\"\nB=inside\"\nC=1";
        assert_eq!(keys(src), ["A", "C"]);
    }

    #[test]
    fn backslash_is_literal_in_single_quotes() {
        assert_eq!(keys("A='C:\\'\nB=1"), ["A", "B"]);
    }

    #[test]
    fn rejects_line_without_equals() {
        let err = parse("A=1\nJUSTAKEY\n").unwrap_err();
        assert_eq!(err.line, 2);
        assert!(err.message.contains("JUSTAKEY"));
    }

    #[test]
    fn rejects_invalid_key() {
        let err = parse("1ABC=x").unwrap_err();
        assert_eq!(err.line, 1);
        assert!(err.message.contains("invalid key"));

        assert!(parse("=x").is_err());
        assert!(parse("A B=x").is_err());
    }

    #[test]
    fn rejects_unterminated_quote() {
        let err = parse("A=1\nB=\"open\nC=2\n").unwrap_err();
        assert_eq!(err.line, 2);
        assert!(err.message.contains("unterminated"));
    }

    #[test]
    fn handles_crlf_line_endings() {
        assert_eq!(keys("A=1\r\nB=2\r\n"), ["A", "B"]);
    }

    #[test]
    fn empty_input() {
        assert!(parse("").unwrap().is_empty());
    }
}
