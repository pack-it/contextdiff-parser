use thiserror::Error;

use crate::specification::TimestampParseError;

/// An error during parsing, at the given line and column.
#[derive(Error, Debug)]
#[error("{kind} at line {line} column {column}")]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub line: u64,
    pub column: u64,
}

/// The errors that could occur during parsing.
#[derive(Error, Debug)]
pub enum ParserErrorKind {
    #[error("Unexpected end of file")]
    UnexpectedEOF,

    #[error("Unexpected end of line")]
    UnexpectedEOL,

    #[error("Expected file header prefix")]
    ExpectedFileHeaderPrefix,

    #[error("Expected tab character to separate path and timestmap in file header")]
    ExpectedTabInFileHeaderPrefix,

    #[error("Expected a non empty file name in file header")]
    EmptyFileNameInHeader,

    #[error("Expected hunk separator")]
    ExpectedHunkSeparator,

    #[error("Expected hunk prefix")]
    ExpectedHunkPrefix,

    #[error("Expected hunk suffix")]
    ExpectedHunkSuffix,

    #[error("Invalid line number in hunk header '{0}'")]
    InvalidHunkLineNumber(std::num::ParseIntError),

    #[error("The start line number of the hunk is higher than the end line number")]
    HunkStartLineAfterEndLine,

    #[error("Expected {expected} lines in the hunk, found {found} lines")]
    HunkHeaderAndLinesMismatch {
        expected: u64,
        found: u64,
    },

    #[error("Invalid line indicator '{0}'")]
    InvalidLineIndicator(char),

    #[error("Expected a space after the line indicator")]
    ExpectedSpaceAfterIndicator,

    #[error("Unable to parse timestamp: '{0}'")]
    TimestampParseError(#[from] TimestampParseError),
}

pub type Result<T> = core::result::Result<T, ParserError>;

impl ParserError {
    /// Creates a new `ParserError`.
    pub const fn new(line: u64, column: u64, kind: ParserErrorKind) -> Self {
        Self { kind, line, column }
    }

    /// Creates a new `ParserError::UnexpectedEOF`.
    pub const fn unexpected_eof(line: u64) -> Self {
        Self {
            kind: ParserErrorKind::UnexpectedEOF,
            line,
            column: 0,
        }
    }
}
