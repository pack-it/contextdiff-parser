//! Parser to parse context diffs.
//! Includes a [`segmenter`] module to split context diff hunks into segments,
//! with each segment being either a pair of changes or context lines.

mod error;
mod iterator;
pub mod segmenter;

#[allow(clippy::module_inception)]
mod parser;

#[cfg(test)]
mod tests;

pub use self::parser::parse_from_str;

pub use self::error::ParserError;
pub use self::error::Result;
