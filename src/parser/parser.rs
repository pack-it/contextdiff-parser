use std::str::FromStr;

use crate::{
    parser::{
        error::{ParserError, ParserErrorKind, Result},
        iterator::LineIterator,
    },
    specification::{ContextDiffFile, FileDiff, FileDiffHeader, HunkHeader, LineValue, LineValueIndicator, LocalDiff, Timestamp},
};

const FROM_FILE_PREFIX: &str = "*** ";
const TO_FILE_PREFIX: &str = "--- ";
const HUNK_SEPARATOR: &str = "***************";
const FROM_HUNK_HEADER_PREFIX: &str = "*** ";
const FROM_HUNK_HEADER_SUFFIX: &str = " ****";
const TO_HUNK_HEADER_PREFIX: &str = "--- ";
const TO_HUNK_HEADER_SUFFIX: &str = " ----";

/// Parses a context diff file from a string.
pub fn parse_from_str(input: &str) -> Result<ContextDiffFile> {
    let mut diffs = Vec::new();
    let mut comment = String::new();

    let mut iterator = LineIterator::from_lines(input);
    while let Some(line) = iterator.peek() {
        // If we have not yet parsed diffs and the line is not a from file header, store comment
        if diffs.is_empty() && !line.starts_with(FROM_FILE_PREFIX) {
            if !comment.is_empty() {
                comment.push('\n');
            }
            comment.push_str(iterator.next().expect("Expected a line here"));
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
    if separator_line != HUNK_SEPARATOR {
        return Err(ParserError::new(
            iterator.index() as u64,
            0,
            ParserErrorKind::ExpectedFileHeaderPrefix,
        ));
    }

    // Parse from hunk
    let from_file_hunk_line = iterator.index() as u64 + 1;
    let from_file_hunk_header = iterator.next().ok_or(ParserError::unexpected_eof(from_file_hunk_line))?;
    let from_file_hunk_header = parse_hunk_header(from_file_hunk_header, from_file_hunk_line, true)?;

    // Parse lines of from file until the to hunk is found
    let mut from_file_lines = Vec::new();
    let line = iterator.index() as u64;
    while !iterator.peek().ok_or(ParserError::unexpected_eof(line))?.starts_with(TO_HUNK_HEADER_PREFIX) {
        let line = iterator.next().expect("Expected a line here");
        from_file_lines.push(parse_line_value(line, iterator.index() as u64)?);
    }

    // Parse to hunk
    let to_file_hunk_line = iterator.index() as u64 + 1;
    let to_file_hunk_header = iterator.next().ok_or(ParserError::unexpected_eof(to_file_hunk_line))?;
    let to_file_hunk_header = parse_hunk_header(to_file_hunk_header, to_file_hunk_line, false)?;

    // Parse lines of to hunk until a new file or hunk separator is found
    let mut to_file_lines = Vec::new();
    let mut only_insertions = true;
    while let Some(next_line) = iterator.peek()
        && !(next_line.starts_with(FROM_FILE_PREFIX) || *next_line == HUNK_SEPARATOR)
    {
        let line = iterator.next().expect("Expected a line here");
        let line_value = parse_line_value(line, iterator.index() as u64)?;
        if !matches!(line_value.indicator, LineValueIndicator::Unchanged | LineValueIndicator::Inserted) {
            only_insertions = false;
        }
        to_file_lines.push(line_value);
    }

    // Check if from hunk contains the expected number of lines
    if from_file_lines.len() as u64 != from_file_hunk_header.expected_hunk_length() && !only_insertions {
        return Err(ParserError::new(
            from_file_hunk_line,
            0,
            ParserErrorKind::HunkHeaderAndLinesMismatch {
                expected: from_file_hunk_header.expected_hunk_length(),
                found: from_file_lines.len() as u64,
            },
        ));
    }

    // Check if to hunk contains the expected number of lines
    if to_file_lines.len() as u64 != to_file_hunk_header.expected_hunk_length() {
        return Err(ParserError::new(
            to_file_hunk_line,
            0,
            ParserErrorKind::HunkHeaderAndLinesMismatch {
                expected: to_file_hunk_header.expected_hunk_length(),
                found: to_file_lines.len() as u64,
            },
        ));
    }

    Ok(LocalDiff {
        from_file_hunk_header,
        to_file_hunk_header,
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

    // Check if path is not empty
    if path.trim().is_empty() {
        return Err(ParserError::new(
            line_num,
            prefix.len() as u64,
            ParserErrorKind::EmptyFileNameInHeader,
        ));
    }

    let modification_time = Timestamp::from_str(timestamp.trim())
        .map_err(|e| ParserError::new(line_num, tab_index as u64 + prefix.len() as u64 + 1, e.into()))?;

    Ok(FileDiffHeader {
        file_path: path.trim().into(),
        modification_time,
    })
}

/// Parses a hunk header from the given line.
/// Checks the prefix and suffix based on the `is_from` variable.
fn parse_hunk_header(line: &str, line_num: u64, is_from: bool) -> Result<HunkHeader> {
    // Check if line PREFIXs with the expected characters
    let prefix = if is_from { FROM_HUNK_HEADER_PREFIX } else { TO_HUNK_HEADER_PREFIX };
    if !line.starts_with(prefix) {
        return Err(ParserError::new(line_num, 0, ParserErrorKind::ExpectedHunkPrefix));
    }

    // Check if line SUFFIXs with the expected characters
    let suffix = if is_from { FROM_HUNK_HEADER_SUFFIX } else { TO_HUNK_HEADER_SUFFIX };
    if !line.ends_with(suffix) {
        let column = (line.len() - suffix.len()) as u64;
        return Err(ParserError::new(line_num, column, ParserErrorKind::ExpectedHunkSuffix));
    }

    // Extract line number value from hunk header
    let value = line
        .strip_prefix(prefix)
        .expect("Expected a line prefix here")
        .strip_suffix(suffix)
        .expect("Expected a line suffix here");

    // Extract line numbers from hunk
    let (start_line, end_line) = match value.contains(',') {
        true => {
            let (start, end) = value.split_once(',').expect("Expected a comma at the line");
            (Some(start), end)
        },
        false => (None, value),
    };

    let start_line_len = start_line.map_or(0, |x| x.len() + 1) as u64;
    let start_line = match start_line {
        Some(line) => {
            Some(line.parse().map_err(|e| ParserError::new(line_num, prefix.len() as u64, ParserErrorKind::InvalidHunkLineNumber(e)))?)
        },
        None => None,
    };

    let end_line = end_line.parse().map_err(|e| {
        ParserError::new(
            line_num,
            prefix.len() as u64 + start_line_len,
            ParserErrorKind::InvalidHunkLineNumber(e),
        )
    })?;

    // Check if start line is before end line
    if let Some(start_line) = start_line {
        if start_line > end_line {
            return Err(ParserError::new(
                line_num,
                prefix.len() as u64,
                ParserErrorKind::HunkStartLineAfterEndLine,
            ));
        }
    }

    Ok(HunkHeader { start_line, end_line })
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_error {
        ($value:expr, $expected_error_kind:pat, line: $expected_line:pat, col: $expected_column:pat) => {
            assert!(matches!(
                $value,
                Err(ParserError {
                    kind: $expected_error_kind,
                    line: $expected_line,
                    column: $expected_column,
                })
            ));
        };
    }

    fn assert_parse_file_diff_header(line: &str, is_from: bool, expected_file_path: &str, expected_timestamp: Timestamp) {
        match parse_file_diff_header(line, 0, is_from) {
            Ok(value) => assert_eq!(
                value,
                FileDiffHeader {
                    file_path: expected_file_path.into(),
                    modification_time: expected_timestamp,
                },
            ),
            Err(e) => {
                panic!(
                    "Expected Ok(FileDiffHeader {{ file_path: {expected_file_path:?}, modification_time: {expected_timestamp:?} }}), got Err({e:?})"
                )
            },
        };
    }

    #[test]
    fn test_file_diff_header() {
        let test_timestamp = Timestamp::from_str("2002-02-21 23:30:39.000000000 -0000").expect("Expected valid timestamp");

        // Test valid parsing of simple headers
        assert_parse_file_diff_header(
            "*** some/path	2002-02-21 23:30:39.000000000 -0000",
            true,
            "some/path",
            test_timestamp.clone(),
        );
        assert_parse_file_diff_header(
            "--- some/path	2002-02-21 23:30:39.000000000 -0000",
            false,
            "some/path",
            test_timestamp.clone(),
        );

        // Test valid parsing of paths with spaces in header
        assert_parse_file_diff_header(
            "*** some/path with spaces	2002-02-21 23:30:39.000000000 -0000",
            true,
            "some/path with spaces",
            test_timestamp.clone(),
        );
        assert_parse_file_diff_header(
            "--- some/path with spaces	2002-02-21 23:30:39.000000000 -0000",
            false,
            "some/path with spaces",
            test_timestamp.clone(),
        );

        // Test valid parsing of POSIX timestamp format in header
        assert_parse_file_diff_header("*** some/path	Thu Feb 21 23:30:39 2002", true, "some/path", test_timestamp.clone());
        assert_parse_file_diff_header("--- some/path	Thu Feb 21 23:30:39 2002", false, "some/path", test_timestamp);

        // Test invalid header prefix
        let value = parse_file_diff_header("some/path	Thu Feb 21 23:30:39 2002", 0, true);
        assert_error!(value, ParserErrorKind::ExpectedFileHeaderPrefix, line: 0, col: 0);
        let value = parse_file_diff_header("*** some/path	Thu Feb 21 23:30:39 2002", 0, false);
        assert_error!(value, ParserErrorKind::ExpectedFileHeaderPrefix, line: 0, col: 0);
        let value = parse_file_diff_header("--- some/path	Thu Feb 21 23:30:39 2002", 0, true);
        assert_error!(value, ParserErrorKind::ExpectedFileHeaderPrefix, line: 0, col: 0);

        // Test no tab in header
        let value = parse_file_diff_header("*** some/path Thu Feb 21 23:30:39 2002", 0, true);
        assert_error!(value, ParserErrorKind::ExpectedTabInFileHeaderPrefix, line: 0, col: 0);

        // Test empty filename
        let value = parse_file_diff_header("*** 	Thu Feb 21 23:30:39 2002", 0, true);
        assert_error!(value, ParserErrorKind::EmptyFileNameInHeader, line: 0, col: 4);

        // Test invalid timestamp
        assert!(matches!(
            parse_file_diff_header("*** some/path	timestamp", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::TimestampParseError(_),
                line: 0,
                column: 14,
            })
        ));

        // Test timestamp with arbitrary text after
        assert!(matches!(
            parse_file_diff_header("*** some/path	Thu Feb 21 23:30:39 2002 text", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::TimestampParseError(_),
                line: 0,
                column: 14,
            })
        ));
    }

    fn assert_parse_hunk_header(line: &str, is_from: bool, expected_start_line: Option<u64>, expected_end_line: u64) {
        match parse_hunk_header(line, 0, is_from) {
            Ok(value) => assert_eq!(
                value,
                HunkHeader {
                    start_line: expected_start_line,
                    end_line: expected_end_line,
                },
            ),
            Err(e) => {
                panic!("Expected Ok(HunkHeader {{ start_line: {expected_start_line:?}, end_line: {expected_end_line:?} }}), got Err({e:?})")
            },
        };
    }

    #[test]
    fn test_hunk_header() {
        // Test valid parsing of simple headers
        assert_parse_hunk_header("*** 10,15 ****", true, Some(10), 15);
        assert_parse_hunk_header("--- 10,15 ----", false, Some(10), 15);
        assert_parse_hunk_header("*** 10 ****", true, None, 10);
        assert_parse_hunk_header("--- 10 ----", false, None, 10);

        // Test invalid header prefix
        let value = parse_hunk_header("*** 10,15 ****", 0, false);
        assert_error!(value, ParserErrorKind::ExpectedHunkPrefix, line: 0, col: 0);
        let value = parse_hunk_header("--- 10,15 ****", 0, false);
        assert_error!(value, ParserErrorKind::ExpectedHunkSuffix, line: 0, col: 9);

        let value = parse_hunk_header("--- 10,15 ----", 0, true);
        assert_error!(value, ParserErrorKind::ExpectedHunkPrefix, line: 0, col: 0);
        let value = parse_hunk_header("*** 10,15 ----", 0, true);
        assert_error!(value, ParserErrorKind::ExpectedHunkSuffix, line: 0, col: 9);

        // Test invalid hunk line number
        assert!(matches!(
            parse_hunk_header("*** a,15 ****", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::InvalidHunkLineNumber(_),
                line: 0,
                column: 4,
            })
        ));
        assert!(matches!(
            parse_hunk_header("*** 10,a ****", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::InvalidHunkLineNumber(_),
                line: 0,
                column: 7,
            })
        ));

        // Test empty hunk line number
        assert!(matches!(
            parse_hunk_header("*** ,15 ****", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::InvalidHunkLineNumber(_),
                line: 0,
                column: 4,
            })
        ));
        assert!(matches!(
            parse_hunk_header("*** 10, ****", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::InvalidHunkLineNumber(_),
                line: 0,
                column: 7,
            })
        ));
        assert!(matches!(
            parse_hunk_header("*** , ****", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::InvalidHunkLineNumber(_),
                line: 0,
                column: 4,
            })
        ));

        // Test start line after end line
        assert!(matches!(
            parse_hunk_header("*** 15,10 ****", 0, true),
            Err(ParserError {
                kind: ParserErrorKind::HunkStartLineAfterEndLine,
                line: 0,
                column: 4,
            })
        ));
    }

    fn assert_parse_line_value(line: &str, expected_line_value: &str, expected_line_indicator: LineValueIndicator) {
        match parse_line_value(line, 0) {
            Ok(value) => assert_eq!(
                value,
                LineValue {
                    line_value: expected_line_value.into(),
                    indicator: expected_line_indicator,
                },
            ),
            Err(e) => {
                panic!(
                    "Expected Ok(LineValue {{ line_value: {expected_line_value:?}, indicator: {expected_line_indicator:?} }}), got Err({e:?})"
                )
            },
        };
    }

    #[test]
    fn test_line_value() {
        // Test valid parsing of simple lines
        assert_parse_line_value("  if (result == 0)", "if (result == 0)", LineValueIndicator::Unchanged);
        assert_parse_line_value("! if (result == 0)", "if (result == 0)", LineValueIndicator::Changed);
        assert_parse_line_value("+ if (result == 0)", "if (result == 0)", LineValueIndicator::Inserted);
        assert_parse_line_value("- if (result == 0)", "if (result == 0)", LineValueIndicator::Deleted);

        // Test valid parsing of empty lines
        assert_parse_line_value("  ", "", LineValueIndicator::Unchanged);
        assert_parse_line_value("! ", "", LineValueIndicator::Changed);
        assert_parse_line_value("+ ", "", LineValueIndicator::Inserted);
        assert_parse_line_value("- ", "", LineValueIndicator::Deleted);

        // Test invalid indicator
        let value = parse_line_value("% if (result == 0)", 0);
        assert_error!(value, ParserErrorKind::InvalidLineIndicator('%'), line: 0, col: 0);

        // Test no space after indicator
        let value = parse_line_value("!if (result == 0)", 0);
        assert_error!(value, ParserErrorKind::ExpectedSpaceAfterIndicator, line: 0, col: 1);

        // Test not enough characters
        assert_error!(parse_line_value("", 0), ParserErrorKind::UnexpectedEOL, line: 0, col: 0);
        assert_error!(parse_line_value("!", 0), ParserErrorKind::UnexpectedEOL, line: 0, col: 1);
    }
}
