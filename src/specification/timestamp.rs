use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, FixedOffset};

/// Represents a timestamp in a context diff file.
#[derive(Debug)]
pub struct Timestamp {
    value: DateTime<FixedOffset>,
}

impl FromStr for Timestamp {
    type Err = chrono::ParseError;

    /// Parses a context diff timestamp from a string, expects a RFC2822 format.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            value: DateTime::parse_from_rfc2822(s)?,
        })
    }
}

impl Display for Timestamp {
    /// Formats a context diff timestamp as string in RFC2822 format.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.to_rfc2822())
    }
}
