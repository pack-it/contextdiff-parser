use crate::specification::timestamp::Timestamp;

/// Represents a context diff file, consisting of the separate diffs for each file.
pub struct ContextDiffFile {
    diffs: Vec<FileDiff>,
}

/// Represents all changes for one file in a context diff file.
pub struct FileDiff {
    from_file: String,
    to_file: String,
    from_modification_time: Timestamp,
    to_modification_time: Timestamp,
    diffs: Vec<LocalDiff>,
}

/// Represents all changes to one local block in a file.
pub struct LocalDiff {
    from_file_line_numbers: String,
    to_file_line_numbers: String,
    from_file_lines: Vec<LineValue>,
    to_file_lines: Vec<LineValue>,
}

/// Represents the value of one line in a local diff.
pub struct LineValue {
    line_value: String,
    indicator: LineValueIndicator,
}

/// Represents all possible indicators of a line in a local diff.
pub enum LineValueIndicator {
    /// No indicator, line is not changed.
    Nothing,

    /// '!' indicator, line was changed.
    Changed,

    /// '+' indicator, line was inserted.
    Inserted,

    /// '-' indicator, line was deleted.
    Deleted,
}
