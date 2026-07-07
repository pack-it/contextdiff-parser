//! A simple Rust library to parse and translate context diff files.
//!
//! contextdiff-parser provides a simple parser for context diff files, a segmenter to split context
//! diff hunks into hunk segments to be used for clear separation between changes and context lines,
//! and a simple translator to translate these context diffs into unified diffs
//!
//! Example
//!
//! A simple context diff to unified diff parser application
//!
//! ```
//! use contextdiff_parser::{parser, translator};
//! use std::{env, fs};
//!
//! fn main() {
//!     let args: Vec<String> = env::args().collect();
//!     let file_name = args.get(1).expect("Expected a file as argument");
//!
//!     let input = fs::read_to_string(file_name).expect("Expected given file to be readable");
//!
//!     match parser::parse_from_str(&input) {
//!         Ok(parsed) => {
//!             let unified_diff = translator::translate_to_unified_diff(parsed);
//!             println!("{unified_diff}");
//!         },
//!         Err(e) => println!("ERROR: {e}"),
//!     }
//! }
//! ```

pub mod parser;
pub mod specification;
pub mod translator;
