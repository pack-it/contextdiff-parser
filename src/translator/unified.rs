use crate::{
    parser::segmenter::{self, HunkSegment},
    specification::{ContextDiffFile, FileDiffHeader, LineValueIndicator, LocalDiff},
};

/// Translates a context diff to a unified diff.
pub fn translate_to_unified_diff(context_diff: ContextDiffFile) -> String {
    let mut string = String::new();

    // Add comments
    string.push_str(&context_diff.comment);
    string.push('\n');

    // Add all file diffs
    for file in context_diff.diffs {
        string.push_str(&translate_file_header(file.from_header, true));
        string.push_str(&translate_file_header(file.to_header, false));

        // Add all hunks
        for diff in file.diffs {
            string.push_str(&translate_hunk(diff));
        }
    }

    string
}

fn translate_file_header(header: FileDiffHeader, is_from: bool) -> String {
    let prefix = if is_from { "---" } else { "+++" };
    let path = header.file_path;
    let timestamp = header.modification_time;

    format!("{prefix} {path}\t{timestamp}\n")
}

fn translate_hunk(hunk: LocalDiff) -> String {
    let mut translated_hunk = String::new();

    let old_start = hunk.from_file_hunk_header.start_line.unwrap_or(hunk.from_file_hunk_header.end_line);
    let new_start = hunk.to_file_hunk_header.start_line.unwrap_or(hunk.to_file_hunk_header.end_line);

    let old_length = hunk.from_file_hunk_header.expected_hunk_length();
    let new_length = hunk.to_file_hunk_header.expected_hunk_length();

    translated_hunk.push_str(&format!("@@ -{old_start},{old_length} +{new_start},{new_length} @@\n"));

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
                        _ => unreachable!(),
                    }
                }
                for line in to {
                    match line.indicator {
                        LineValueIndicator::Changed | LineValueIndicator::Inserted => {
                            translated_hunk.push_str(&format_insertion(&line.line_value));
                        },
                        _ => unreachable!(),
                    }
                }
            },
        }
    }

    translated_hunk
}

fn format_context(value: &str) -> String {
    format!(" {value}\n")
}

fn format_deletion(value: &str) -> String {
    format!("-{value}\n")
}

fn format_insertion(value: &str) -> String {
    format!("+{value}\n")
}
