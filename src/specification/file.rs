use crate::specification::timestamp::Timestamp;

/// Represents a context diff file, consisting of the separate diffs for each file.
#[derive(Debug)]
pub struct ContextDiffFile {
    pub comment: String,
    pub diffs: Vec<FileDiff>,
}

/// Represents all changes for one file in a context diff file.
#[derive(Debug)]
pub struct FileDiff {
    pub from_header: FileDiffHeader,
    pub to_header: FileDiffHeader,
    pub hunks: Vec<Hunk>,
}

/// Represents a header of a file diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDiffHeader {
    pub file_path: String,
    pub modification_time: Timestamp,
}

/// Represents a hunk, containing all changes to one local block in a file.
#[derive(Debug)]
pub struct Hunk {
    pub from_file_header: HunkHeader,
    pub from_file_lines: Vec<LineValue>,
    pub to_file_header: HunkHeader,
    pub to_file_lines: Vec<LineValue>,
}

/// Represents a header of a hunk of a local diff.
#[derive(Debug, PartialEq, Eq)]
pub struct HunkHeader {
    pub start_line: Option<u64>,
    pub end_line: u64,
}

/// Represents the value of one line in a local diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineValue {
    pub line_value: String,
    pub indicator: LineValueIndicator,
}

/// Represents all possible indicators of a line in a local diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineValueIndicator {
    /// ' ' indicator, line was not changed.
    Unchanged,

    /// '!' indicator, line was changed.
    Changed,

    /// '+' indicator, line was inserted.
    Inserted,

    /// '-' indicator, line was deleted.
    Deleted,
}

impl HunkHeader {
    /// Calculates the expected length of the hunk.
    pub const fn expected_hunk_length(&self) -> u64 {
        match self.start_line {
            Some(start_line) => self.end_line - start_line + 1,
            None => 1,
        }
    }
}

impl LineValue {
    /// Creates a new `LineValue` with the given value and indicator
    pub fn new(line_value: impl Into<String>, indicator: LineValueIndicator) -> Self {
        Self {
            line_value: line_value.into(),
            indicator,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expected_hunk_length() {
        let hunk_header = HunkHeader {
            start_line: Some(10),
            end_line: 15,
        };

        assert_eq!(hunk_header.expected_hunk_length(), 6);

        let hunk_header = HunkHeader {
            start_line: None,
            end_line: 15,
        };

        assert_eq!(hunk_header.expected_hunk_length(), 1);
    }
}
