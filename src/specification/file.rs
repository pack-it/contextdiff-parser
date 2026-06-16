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
    pub diffs: Vec<LocalDiff>,
}

/// Represents a header of a file diff.
#[derive(Debug)]
pub struct FileDiffHeader {
    pub file_path: String,
    pub modification_time: Timestamp,
}

/// Represents all changes to one local block in a file.
#[derive(Debug)]
pub struct LocalDiff {
    pub from_file_hunk: Hunk,
    pub from_file_lines: Vec<LineValue>,
    pub to_file_hunk: Hunk,
    pub to_file_lines: Vec<LineValue>,
}

/// Represents a hunk of a local diff.
#[derive(Debug)]
pub struct Hunk {
    pub line_numbers: String,
}

/// Represents the value of one line in a local diff.
#[derive(Debug)]
pub struct LineValue {
    pub line_value: String,
    pub indicator: LineValueIndicator,
}

/// Represents all possible indicators of a line in a local diff.
#[derive(Debug)]
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
