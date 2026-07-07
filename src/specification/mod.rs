//! Specification of the context diff format.
//! The specification of the parts in a context diff file is defined in [`file`],
//! the specification of the timestamps used in context diffs is defined in [`timestamp`].

mod file;
mod timestamp;

pub use self::file::ContextDiffFile;
pub use self::file::FileDiff;
pub use self::file::FileDiffHeader;
pub use self::file::Hunk;
pub use self::file::HunkHeader;
pub use self::file::LineValue;
pub use self::file::LineValueIndicator;

pub use self::timestamp::Timestamp;
pub use self::timestamp::TimestampParseError;
