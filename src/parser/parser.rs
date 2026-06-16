use std::str::FromStr;

use crate::{
    parser::{
        error::{ParserError, ParserErrorKind, Result},
        iterator::LineIterator,
    },
    specification::{ContextDiffFile, FileDiff, FileDiffHeader, Hunk, LineValue, LineValueIndicator, LocalDiff, Timestamp},
};

const FROM_FILE_PREFIX: &str = "*** ";
const TO_FILE_PREFIX: &str = "--- ";
const HUNK_SEPARATOR: &str = "***************";
const FROM_HUNK_PREFIX: &str = "*** ";
const FROM_HUNK_SUFFIX: &str = " ****";
const TO_HUNK_PREFIX: &str = "--- ";
const TO_HUNK_SUFFIX: &str = " ----";

/// Parses a context diff file from a string.
pub fn parse_from_str(input: &str) -> Result<ContextDiffFile> {
    let mut diffs = Vec::new();
    let mut comment = String::new();

    let mut iterator = LineIterator::from_lines(input);
    while let Some(line) = iterator.peek() {
        // If we have not yet parsed diffs and the line is not a from file header, store comment
        if diffs.is_empty() && !line.starts_with(FROM_FILE_PREFIX) {
            comment.push_str(iterator.next().expect("Expected a line here"));
            comment.push('\n');
            continue;
        }

        diffs.push(parse_next_file_diff(&mut iterator)?);
    }

    Ok(ContextDiffFile { comment, diffs })
}

/// Parses the next file diff from the given iterator.
fn parse_next_file_diff(iterator: &mut LineIterator) -> Result<FileDiff> {
    // Parse file headers
    let from_header = iterator.next().ok_or(ParserError::unexpected_eof(iterator.index() as u64))?;
    let from_header = parse_file_diff_header(from_header, iterator.index() as u64, true)?;
    let to_header = iterator.next().ok_or(ParserError::unexpected_eof(iterator.index() as u64))?;
    let to_header = parse_file_diff_header(to_header, iterator.index() as u64, false)?;

    // Parse local diffs of the file until a new file is found
    let mut diffs = Vec::new();
    while let Some(next_line) = iterator.peek()
        && !next_line.starts_with(FROM_FILE_PREFIX)
    {
        diffs.push(parse_next_local_diff(iterator)?);
    }

    Ok(FileDiff {
        from_header,
        to_header,
        diffs,
    })
}

/// Parses the next local diff from the given iterator
fn parse_next_local_diff(iterator: &mut LineIterator) -> Result<LocalDiff> {
    // Check if next line is the expected separator
    let separator_line = iterator.next().ok_or(ParserError::unexpected_eof(iterator.index() as u64))?;
    if separator_line.trim() != HUNK_SEPARATOR {
        return Err(ParserError::new(
            iterator.index() as u64,
            0,
            ParserErrorKind::ExpectedFileHeaderPrefix,
        ));
    }

    // Parse from hunk
    let from_file_hunk = iterator.next().ok_or(ParserError::unexpected_eof(iterator.index() as u64))?;
    let from_file_hunk = parse_hunk(from_file_hunk, iterator.index() as u64, true)?;

    // Parse lines of from file until the to hunk is found
    let mut from_file_lines = Vec::new();
    let line = iterator.index() as u64;
    while !iterator.peek().ok_or(ParserError::unexpected_eof(line))?.starts_with(TO_HUNK_PREFIX) {
        let line = iterator.next().expect("Expected a line here");
        from_file_lines.push(parse_line_value(line, iterator.index() as u64)?);
    }

    // Parse to hunk
    let to_file_hunk = iterator.next().ok_or(ParserError::unexpected_eof(iterator.index() as u64))?;
    let to_file_hunk = parse_hunk(to_file_hunk, iterator.index() as u64, false)?;

    // Parse lines of to hunk until a new file or hunk separator is found
    let mut to_file_lines = Vec::new();
    while let Some(next_line) = iterator.peek()
        && !(next_line.starts_with(FROM_FILE_PREFIX) || next_line.trim() == HUNK_SEPARATOR)
    {
        let line = iterator.next().expect("Expected a line here");
        to_file_lines.push(parse_line_value(line, iterator.index() as u64)?);
    }

    Ok(LocalDiff {
        from_file_hunk,
        to_file_hunk,
        from_file_lines,
        to_file_lines,
    })
}

/// Parses a file diff header from the given line.
/// Checks the prefix based on the `is_from` variable.
fn parse_file_diff_header(line: &str, line_num: u64, is_from: bool) -> Result<FileDiffHeader> {
    // Check if file PREFIXs with the correct prefix
    let prefix = if is_from { FROM_FILE_PREFIX } else { TO_FILE_PREFIX };
    if !line.starts_with(prefix) {
        return Err(ParserError::new(line_num, 0, ParserErrorKind::ExpectedFileHeaderPrefix));
    }

    // Split path and timestamp from header
    let value = line.strip_prefix(prefix).expect("Expected a line prefix here");
    let tab_index = value.find('\t').ok_or(ParserError::new(line_num, 0, ParserErrorKind::ExpectedTabInFileHeaderPrefix))?;
    let (path, timestamp) = value.split_at(tab_index);

    let modification_time = Timestamp::from_str(timestamp.trim())
        .map_err(|e| ParserError::new(line_num, tab_index as u64 + prefix.len() as u64 + 1, e.into()))?;

    Ok(FileDiffHeader {
        file_path: path.into(),
        modification_time,
    })
}

/// Parses a hunk from the given line.
/// Checks the prefix and suffix based on the `is_from` variable.
fn parse_hunk(line: &str, line_num: u64, is_from: bool) -> Result<Hunk> {
    // Check if line PREFIXs with the expected characters
    let prefix = if is_from { FROM_HUNK_PREFIX } else { TO_HUNK_PREFIX };
    if !line.starts_with(prefix) {
        return Err(ParserError::new(line_num, 0, ParserErrorKind::ExpectedHunkPrefix));
    }

    // Check if line SUFFIXs with the expected characters
    let suffix = if is_from { FROM_HUNK_SUFFIX } else { TO_HUNK_SUFFIX };
    if !line.ends_with(suffix) {
        let column = (line.len() - suffix.len()) as u64;
        return Err(ParserError::new(line_num, column, ParserErrorKind::ExpectedHunkSuffix));
    }

    // Extract line numbers from hunk
    let value = line
        .strip_prefix(prefix)
        .expect("Expected a line prefix here")
        .strip_suffix(suffix)
        .expect("Expected a line suffix here");

    Ok(Hunk {
        line_numbers: value.into(),
    })
}

/// Parses a line value from the given line.
fn parse_line_value(line: &str, line_num: u64) -> Result<LineValue> {
    let mut chars = line.chars();

    // Match the indicator of the line
    let indicator_char = chars.next().ok_or(ParserError::new(line_num, 0, ParserErrorKind::UnexpectedEOL))?;
    let indicator = match indicator_char {
        ' ' => LineValueIndicator::Unchanged,
        '!' => LineValueIndicator::Changed,
        '+' => LineValueIndicator::Inserted,
        '-' => LineValueIndicator::Deleted,
        indicator_char => return Err(ParserError::new(line_num, 0, ParserErrorKind::InvalidLineIndicator(indicator_char))),
    };

    // Check if second character of line is a space as expected
    let second_char = chars.next().ok_or(ParserError::new(line_num, 1, ParserErrorKind::UnexpectedEOL))?;
    if second_char != ' ' {
        return Err(ParserError::new(line_num, 1, ParserErrorKind::ExpectedSpaceAfterIndicator));
    }

    // Extract line value from line
    let line_value = chars.as_str();

    Ok(LineValue {
        line_value: line_value.into(),
        indicator,
    })
}
