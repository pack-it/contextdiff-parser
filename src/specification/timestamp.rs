use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use thiserror::Error;

/// Represents a timestamp in a context diff file.
#[derive(Debug)]
pub struct Timestamp {
    value: DateTime<FixedOffset>,
}

/// The errors that could occur during parsing of timestamps.
#[derive(Error, Debug)]
pub enum TimestampParseError {
    #[error("timestamp is empty")]
    TimestampIsEmpty,

    #[error("{0}")]
    ChronoParseError(chrono::ParseError),
}

impl FromStr for Timestamp {
    type Err = TimestampParseError;

    /// Parses a context diff timestamp from a string.
    /// Expects either the GNU format or the traditional POSIX format:
    /// GNU: 2002-02-21 23:30:39.942229878 -0800
    /// POSIX: Thu Feb 21 23:30:39 2002
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(TimestampParseError::TimestampIsEmpty);
        }

        // Check if the first character is a-z, which would indicate a POSIX timestamp
        if s.chars().next().expect("Expected a first character").is_ascii_alphabetic() {
            let naive = NaiveDateTime::parse_from_str(s, "%a %b %e %H:%M:%S %Y").map_err(|e| TimestampParseError::ChronoParseError(e))?;
            return Ok(Self {
                value: FixedOffset::east_opt(0).expect("Expected 0 to be a valid offset").from_utc_datetime(&naive),
            });
        }

        // Assume GNU timestamp
        Ok(Self {
            value: DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f %z").map_err(|e| TimestampParseError::ChronoParseError(e))?,
        })
    }
}

impl Display for Timestamp {
    /// Formats a context diff timestamp as string in GNU format (2002-02-21 23:30:39.942229878 -0800).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.format("%Y-%m-%d %H:%M:%S%.f %z"))
    }
}
