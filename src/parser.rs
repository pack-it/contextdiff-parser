use std::{iter::Peekable, str::FromStr};

use crate::specification::{ContextDiffFile, FileDiff, FileDiffHeader, Hunk, LineValue, LineValueIndicator, LocalDiff, Timestamp};

const FROM_FILE_PREFIX: &str = "*** ";
const TO_FILE_PREFIX: &str = "--- ";
const HUNK_SEPARATOR: &str = "***************";
const FROM_HUNK_PREFIX: &str = "*** ";
const FROM_HUNK_SUFFIX: &str = " ****";
const TO_HUNK_PREFIX: &str = "--- ";
const TO_HUNK_SUFFIX: &str = " ----";

/// Parses a context diff file from a string.
pub fn parse_from_str(input: &str) -> Option<ContextDiffFile> {
    let mut diffs = Vec::new();
    let mut comment = String::new();

    let mut iterator: Peekable<_> = input.split('\n').peekable();
    while let Some(line) = iterator.peek() {
        // If we have not yet parsed diffs and the line is not a from file header, store comment
        if diffs.is_empty() && !line.starts_with(FROM_FILE_PREFIX) {
            comment.push_str(iterator.next().expect("Expected a line here"));
            comment.push('\n');
            continue;
        }

        diffs.push(parse_next_file_diff(&mut iterator)?);
    }

    Some(ContextDiffFile { comment, diffs })
}

/// Parses the next file diff from the given iterator.
fn parse_next_file_diff<'a>(iterator: &mut Peekable<impl Iterator<Item = &'a str>>) -> Option<FileDiff> {
    let from_header = iterator.next()?;
    let to_header = iterator.next()?;

    // Parse file headers
    let from_header = parse_file_diff_header(from_header, true)?;
    let to_header = parse_file_diff_header(to_header, false)?;

    // Parse local diffs of the file until a new file is found
    let mut diffs = Vec::new();
    while let Some(next_line) = iterator.peek()
        && !next_line.starts_with(FROM_FILE_PREFIX)
    {
        diffs.push(parse_next_local_diff(iterator)?);
    }

    Some(FileDiff {
        from_header,
        to_header,
        diffs,
    })
}

/// Parses the next local diff from the given iterator
fn parse_next_local_diff<'a>(iterator: &mut Peekable<impl Iterator<Item = &'a str>>) -> Option<LocalDiff> {
    // Check if next line is the expected separator
    if iterator.next()?.trim() != HUNK_SEPARATOR {
        return None;
    }

    // Parse from hunk
    let from_file_hunk = iterator.next()?;
    let from_file_hunk = parse_hunk(from_file_hunk, true)?;

    // Parse lines of from file until the to hunk is found
    let mut from_file_lines = Vec::new();
    while !iterator.peek()?.starts_with(TO_HUNK_PREFIX) {
        let line = iterator.next().expect("Expected a line here");
        from_file_lines.push(parse_line_value(line)?);
    }

    // Parse to hunk
    let to_file_hunk = iterator.next()?;
    let to_file_hunk = parse_hunk(to_file_hunk, false)?;

    // Parse lines of to hunk until a new file or hunk separator is found
    let mut to_file_lines = Vec::new();
    while let Some(next_line) = iterator.peek()
        && !(next_line.starts_with(FROM_FILE_PREFIX) || next_line.trim() == HUNK_SEPARATOR)
    {
        let line = iterator.next().expect("Expected a line here");
        to_file_lines.push(parse_line_value(line)?);
    }

    Some(LocalDiff {
        from_file_hunk,
        to_file_hunk,
        from_file_lines,
        to_file_lines,
    })
}

/// Parses a file diff header from the given line.
/// Checks the prefix based on the `is_from` variable.
fn parse_file_diff_header(line: &str, is_from: bool) -> Option<FileDiffHeader> {
    // Check if file PREFIXs with the correct prefix
    let start = if is_from { FROM_FILE_PREFIX } else { TO_FILE_PREFIX };
    if !line.starts_with(start) {
        return None;
    }

    // Split path and timestamp from header
    let value = &line[4..];
    let (path, timestamp) = value.split_once('\t')?;

    let modification_time = Timestamp::from_str(timestamp.trim()).unwrap();

    Some(FileDiffHeader {
        file_path: path.into(),
        modification_time,
    })
}

/// Parses a hunk from the given line.
/// Checks the prefix and suffix based on the `is_from` variable.
fn parse_hunk(line: &str, is_from: bool) -> Option<Hunk> {
    // Check if line PREFIXs with the expected characters
    let start = if is_from { FROM_HUNK_PREFIX } else { TO_HUNK_PREFIX };
    if !line.starts_with(start) {
        return None;
    }

    // Check if line SUFFIXs with the expected characters
    let end = if is_from { FROM_HUNK_SUFFIX } else { TO_HUNK_SUFFIX };
    if !line.ends_with(end) {
        return None;
    }

    // Extract line numbers from hunk
    let value = &line[4..line.len() - 5];

    Some(Hunk {
        line_numbers: value.into(),
    })
}

/// Parses a line value from the given line.
fn parse_line_value(line: &str) -> Option<LineValue> {
    // Match the indicator of the line
    let indicator = match line.chars().nth(0)? {
        ' ' => LineValueIndicator::Unchanged,
        '!' => LineValueIndicator::Changed,
        '+' => LineValueIndicator::Inserted,
        '-' => LineValueIndicator::Deleted,
        _ => return None,
    };

    // Check if second character of line is a space as expected
    if line.chars().nth(1)? != ' ' {
        return None;
    }

    // Extract line value from line
    let line_value = &line[2..];

    Some(LineValue {
        line_value: line_value.into(),
        indicator,
    })
}
