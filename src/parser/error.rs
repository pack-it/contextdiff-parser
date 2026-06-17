use thiserror::Error;

use crate::specification::TimestampParseError;

/// An error during parsing, at the given line and column.
#[derive(Error, Debug)]
#[error("{kind} at line {line} column {column}")]
pub struct ParserError {
    kind: ParserErrorKind,
    line: u64,
    column: u64,
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

    #[error("Expected hunk separator")]
    ExpectedHunkSeparator,

    #[error("Expected hunk prefix")]
    ExpectedHunkPrefix,

    #[error("Expected hunk suffix")]
    ExpectedHunkSuffix,

    #[error("Invalid line indicator '{0}'")]
    InvalidLineIndicator(char),

    #[error("Expected a space after the line indicator")]
    ExpectedSpaceAfterIndicator,

    #[error("Unable to parse timestamp: '{0}'")]
    TimestampParseError(#[from] TimestampParseError),
}

pub type Result<T> = core::result::Result<T, ParserError>;

impl ParserError {
    /// Creates a new ParserError.
    pub fn new(line: u64, column: u64, kind: ParserErrorKind) -> Self {
        Self { kind, line, column }
    }

    /// Creates a new UnexpectedEOF ParserError.
    pub fn unexpected_eof(line: u64) -> Self {
        Self {
            kind: ParserErrorKind::UnexpectedEOF,
            line,
            column: 0,
        }
    }
}
