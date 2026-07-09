use crate::{
    parser::segmenter::{self, HunkSegment},
    specification::{ContextDiffFile, FileDiffHeader, Hunk, HunkHeader, LineValueIndicator},
};

/// Translates a context diff to a unified diff.
///
/// # Example
///
/// ```
/// use contextdiff_parser::{parser, translator};
///
/// let input = "Some comment
/// *** file	2026-06-19 04:02:26.103103072 +0200
/// --- file	2026-06-19 08:29:42.921015162 +0200
/// ***************
/// *** 1,2 ****
/// --- 1,3 ----
///   This is some file
///   With two lines
/// + And even a third line!
/// ";
///
/// match parser::parse_from_str(input) {
///     Ok(parsed) => {
///         let unified_diff = translator::translate_to_unified_diff(parsed);
///         println!("{unified_diff}");
///     },
///     Err(e) => println!("ERROR: {e}"),
/// }
/// ```
pub fn translate_to_unified_diff(context_diff: ContextDiffFile) -> String {
    let mut string = String::new();

    // Add comments
    if !context_diff.comment.is_empty() {
        string.push_str(&context_diff.comment);
        string.push('\n');
    }

    // Add all file diffs
    for file in context_diff.diffs {
        string.push_str(&translate_file_header(file.from_header, true));
        string.push_str(&translate_file_header(file.to_header, false));

        // Add all hunks
        for hunk in file.hunks {
            string.push_str(&translate_hunk(hunk));
        }
    }

    string
}

/// Translates a file diff header to unified diff format.
fn translate_file_header(header: FileDiffHeader, is_from: bool) -> String {
    let prefix = if is_from { "---" } else { "+++" };
    let path = header.file_path;
    let timestamp = header.modification_time;

    format!("{prefix} {path}\t{timestamp}\n")
}

/// Translates a hunk to unified diff format.
fn translate_hunk(hunk: Hunk) -> String {
    let mut translated_hunk = String::new();

    // Translate hunk header
    translated_hunk.push_str(&format_hunk_header(&hunk.from_file_header, &hunk.to_file_header));

    // Translate hunk segments
    for segment in segmenter::split_hunk(hunk) {
        match segment {
            HunkSegment::Context(lines) => {
                for line in lines {
                    translated_hunk.push_str(&format_context(&line.line_value));
                }
            },
            HunkSegment::Change { from, to } => {
                for line in from {
                    match line.indicator {
                        LineValueIndicator::Changed | LineValueIndicator::Deleted => {
                            translated_hunk.push_str(&format_deletion(&line.line_value));
                        },

                        // Other indicators are not valid here, so we ignore them if they somehow turned up
                        _ => continue,
                    }
                }
                for line in to {
                    match line.indicator {
                        LineValueIndicator::Changed | LineValueIndicator::Inserted => {
                            translated_hunk.push_str(&format_insertion(&line.line_value));
                        },

                        // Other indicators are not valid here, so we ignore them if they somehow turned up
                        _ => continue,
                    }
                }
            },
        }
    }

    translated_hunk
}

// Formats the from and to headers from context format in a single unified diff format hunk header.
fn format_hunk_header(from_header: &HunkHeader, to_header: &HunkHeader) -> String {
    let old_start = from_header.start_line.unwrap_or(from_header.end_line);
    let new_start = to_header.start_line.unwrap_or(to_header.end_line);

    let old_length = from_header.expected_hunk_length();
    let new_length = to_header.expected_hunk_length();

    format!("@@ -{old_start},{old_length} +{new_start},{new_length} @@\n")
}

/// Formats a context line in unified diff format.
fn format_context(value: &str) -> String {
    format!(" {value}\n")
}

/// Formats a deletion line in unified diff format.
fn format_deletion(value: &str) -> String {
    format!("-{value}\n")
}

/// Formats an insertion line in unified diff format.
fn format_insertion(value: &str) -> String {
    format!("+{value}\n")
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::specification::{FileDiff, HunkHeader, LineValue, Timestamp};

    use super::*;

    #[test]
    fn test_translate_file_header() {
        let header = FileDiffHeader {
            file_path: "some test path".into(),
            modification_time: Timestamp::from_str("2002-02-21 23:30:39.192052015 -0000").expect("Expected valid timestamp"),
        };

        assert_eq!(
            translate_file_header(header.clone(), true),
            "--- some test path\t2002-02-21 23:30:39.192052015 +0000\n"
        );

        assert_eq!(
            translate_file_header(header, false),
            "+++ some test path\t2002-02-21 23:30:39.192052015 +0000\n"
        );
    }

    #[test]
    fn test_format_context() {
        assert_eq!(format_context("this is a test line"), " this is a test line\n");
        assert_eq!(format_context(" this is a test line"), "  this is a test line\n");
        assert_eq!(format_context(""), " \n");
    }

    #[test]
    fn test_format_deletion() {
        assert_eq!(format_deletion("this is a test line"), "-this is a test line\n");
        assert_eq!(format_deletion("-this is a test line"), "--this is a test line\n");
        assert_eq!(format_deletion(""), "-\n");
    }

    #[test]
    fn test_format_insertion() {
        assert_eq!(format_insertion("this is a test line"), "+this is a test line\n");
        assert_eq!(format_insertion("+this is a test line"), "++this is a test line\n");
        assert_eq!(format_insertion(""), "+\n");
    }

    #[test]
    fn test_format_hunk_header() {
        let from_header = HunkHeader {
            start_line: Some(2),
            end_line: 4,
        };
        let to_header = HunkHeader {
            start_line: Some(2),
            end_line: 4,
        };

        assert_eq!(format_hunk_header(&from_header, &to_header), "@@ -2,3 +2,3 @@\n");

        let from_header = HunkHeader {
            start_line: None,
            end_line: 4,
        };
        let to_header = HunkHeader {
            start_line: Some(2),
            end_line: 4,
        };

        assert_eq!(format_hunk_header(&from_header, &to_header), "@@ -4,1 +2,3 @@\n");
    }

    #[test]
    fn test_translate_hunk() {
        let hunk = Hunk {
            from_file_header: HunkHeader {
                start_line: Some(2),
                end_line: 4,
            },
            from_file_lines: vec![
                LineValue::new("test line 1", LineValueIndicator::Unchanged),
                LineValue::new("deleted line 1", LineValueIndicator::Deleted),
                LineValue::new("deleted line 2", LineValueIndicator::Deleted),
            ],
            to_file_header: HunkHeader {
                start_line: None,
                end_line: 2,
            },
            to_file_lines: vec![LineValue::new("test line 1", LineValueIndicator::Unchanged)],
        };

        assert_eq!(
            translate_hunk(hunk),
            "@@ -2,3 +2,1 @@\n test line 1\n-deleted line 1\n-deleted line 2\n"
        );
    }

    #[test]
    fn test_translate_full_file() {
        let hunk = Hunk {
            from_file_header: HunkHeader {
                start_line: Some(2),
                end_line: 4,
            },
            from_file_lines: vec![
                LineValue::new("test line 1", LineValueIndicator::Unchanged),
                LineValue::new("deleted line 1", LineValueIndicator::Deleted),
                LineValue::new("deleted line 2", LineValueIndicator::Deleted),
            ],
            to_file_header: HunkHeader {
                start_line: None,
                end_line: 2,
            },
            to_file_lines: vec![LineValue::new("test line 1", LineValueIndicator::Unchanged)],
        };

        let file = ContextDiffFile {
            comment: "\ntest comments\nwith\nthree lines".into(),
            diffs: vec![FileDiff {
                from_header: FileDiffHeader {
                    file_path: "some test path".into(),
                    modification_time: Timestamp::from_str("2002-02-21 23:30:39.192052015 -0000").expect("Expected valid timestamp"),
                },
                to_header: FileDiffHeader {
                    file_path: "some other test path".into(),
                    modification_time: Timestamp::from_str("2005-02-26 23:31:39.192255025 -0000").expect("Expected valid timestamp"),
                },
                hunks: vec![hunk],
            }],
        };

        assert_eq!(
            translate_to_unified_diff(file),
            "
test comments
with
three lines
--- some test path\t2002-02-21 23:30:39.192052015 +0000
+++ some other test path\t2005-02-26 23:31:39.192255025 +0000
@@ -2,3 +2,1 @@
 test line 1
-deleted line 1
-deleted line 2
"
        );
    }
}
