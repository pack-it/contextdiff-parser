use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use thiserror::Error;

/// Represents a timestamp in a context diff file.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// GNU: 2002-02-21 23:30:39.942229878 -0800 (Note that the fractional second part can be omitted)
    /// POSIX: Thu Feb 21 23:30:39 2002
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(TimestampParseError::TimestampIsEmpty);
        }

        // Check if the first character is a-z, which would indicate a POSIX timestamp
        if s.chars().next().expect("Expected a first character").is_ascii_alphabetic() {
            let naive = NaiveDateTime::parse_from_str(s, "%a %b %e %H:%M:%S %Y").map_err(TimestampParseError::ChronoParseError)?;
            return Ok(Self {
                value: FixedOffset::east_opt(0).expect("Expected 0 to be a valid offset").from_utc_datetime(&naive),
            });
        }

        // Assume GNU timestamp
        Ok(Self {
            value: DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f %z").map_err(TimestampParseError::ChronoParseError)?,
        })
    }
}

impl Display for Timestamp {
    /// Formats a context diff timestamp as string in GNU format (2002-02-21 23:30:39.942229878 -0800).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.format("%Y-%m-%d %H:%M:%S%.f %z"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_timestamps() {
        // Test GNU timestamps
        let correct_timestamp = Timestamp {
            value: DateTime::from_timestamp(1014363039, 942229878).expect("Expected valid timestamp").fixed_offset(),
        };
        match Timestamp::from_str("2002-02-21 23:30:39.942229878 -0800") {
            Ok(timestamp) => assert_eq!(timestamp, correct_timestamp),
            Err(e) => panic!("Expected Ok(Timestamp {{ value: 2002-02-21T23:30:39-08:00 }}), got Err({e:?})"),
        }

        let correct_timestamp = Timestamp {
            value: DateTime::from_timestamp(1014363039, 0).expect("Expected valid timestamp").fixed_offset(),
        };
        match Timestamp::from_str("2002-02-21 23:30:39 -0800") {
            Ok(timestamp) => assert_eq!(timestamp, correct_timestamp),
            Err(e) => panic!("Expected Ok(Timestamp {{ value: 2002-02-21T23:30:39-08:00 }}), got Err({e:?})"),
        }

        // Test POSIX timestamps
        let correct_timestamp = Timestamp {
            value: DateTime::from_timestamp(1014334239, 0).expect("Expected valid timestamp").fixed_offset(),
        };
        match Timestamp::from_str("Thu Feb 21 23:30:39 2002") {
            Ok(timestamp) => assert_eq!(timestamp, correct_timestamp),
            Err(e) => panic!("Expected Ok(Timestamp {{ value: 2002-02-21T23:30:39-00:00 }}), got Err({e:?})"),
        }
    }

    #[test]
    fn test_invalid_timestamps() {
        // Test empty timestamp
        assert!(matches!(Timestamp::from_str(""), Err(TimestampParseError::TimestampIsEmpty)));

        // Test invalid format
        assert!(matches!(
            Timestamp::from_str("2002-02-21 23:30:39.942229878"),
            Err(TimestampParseError::ChronoParseError(_))
        ));
        assert!(matches!(
            Timestamp::from_str("2002-02-21 23:30:39. -0800"),
            Err(TimestampParseError::ChronoParseError(_))
        ));

        // Test invalid characters
        assert!(matches!(
            Timestamp::from_str("some invalid timestamp string"),
            Err(TimestampParseError::ChronoParseError(_))
        ));
        assert!(matches!(
            Timestamp::from_str("2002-02-21 23:30:39.942229878 -0800 invalid"),
            Err(TimestampParseError::ChronoParseError(_))
        ));
    }

    #[test]
    fn test_timestamp_formatting() {
        let timestamp = Timestamp {
            value: DateTime::from_timestamp(1014363039, 942229878).expect("Expected valid timestamp").fixed_offset(),
        };

        assert_eq!(timestamp.to_string(), "2002-02-22 07:30:39.942229878 +0000");
    }
}
