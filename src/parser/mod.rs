mod error;
mod iterator;
pub mod segmenter;

#[allow(clippy::module_inception)]
mod parser;

#[cfg(test)]
mod tests;

pub use self::parser::parse_from_str;
