//! Translator to translate context diffs into another format.
//! Currently only translation to unified diffs is implemented, see [`translate_to_unified_diff`]

mod unified;

pub use self::unified::translate_to_unified_diff;
