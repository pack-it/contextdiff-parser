use crate::specification::{LineValue, LineValueIndicator, LocalDiff};

/// Represents a segment of a hunk, consisting of either context lines or change lines.
#[derive(Debug)]
pub enum HunkSegment {
    Context(Vec<LineValue>),
    Change {
        from: Vec<LineValue>,
        to: Vec<LineValue>,
    },
}

/// Splits a hunk into hunk segments.
pub fn split_hunk(hunk: LocalDiff) -> Vec<HunkSegment> {
    match hunk.from_file_lines.is_empty() {
        true => split_only_to_hunk(hunk),
        false => split_to_and_from_hunk(hunk),
    }
}

fn split_only_to_hunk(hunk: LocalDiff) -> Vec<HunkSegment> {
    let mut blocks = Vec::new();

    let mut sync = false;
    let mut current_block = Vec::new();

    for to_line in hunk.to_file_lines {
        if matches!(to_line.indicator, LineValueIndicator::Unchanged) {
            // Store change block
            if !sync && !current_block.is_empty() {
                blocks.push(HunkSegment::Change {
                    from: Vec::new(),
                    to: current_block,
                });
                current_block = Vec::new();
            }

            // Add line to context block
            sync = true;
            current_block.push(to_line.clone());
            continue;
        }

        // Store context block
        if sync && !current_block.is_empty() {
            blocks.push(HunkSegment::Context(current_block));
            current_block = Vec::new();
        }

        // Add line to change block
        sync = false;
        current_block.push(to_line.clone());
    }

    // Add last block
    if !current_block.is_empty() {
        match sync {
            true => blocks.push(HunkSegment::Context(current_block)),
            false => blocks.push(HunkSegment::Change {
                from: Vec::new(),
                to: current_block,
            }),
        }
    }

    blocks
}

fn split_to_and_from_hunk(hunk: LocalDiff) -> Vec<HunkSegment> {
    let mut blocks = Vec::new();

    let mut sync = false;
    let mut current_block_from = Vec::new();
    let mut current_block_to = Vec::new();

    let mut from_lines = hunk.from_file_lines.iter().peekable();
    let mut to_lines = hunk.to_file_lines.iter().peekable();

    loop {
        let from_line = from_lines.peek();
        let to_line = to_lines.peek();

        // Check both lines available
        if let Some(from_line) = from_line
            && let Some(to_line) = to_line
        {
            // Detect sync block
            if from_line == to_line {
                // Store change block
                if !sync && (!current_block_from.is_empty() || !current_block_to.is_empty()) {
                    blocks.push(HunkSegment::Change {
                        from: current_block_from,
                        to: current_block_to,
                    });
                    current_block_from = Vec::new();
                    current_block_to = Vec::new();
                }

                // Add lines to context block
                sync = true;
                current_block_from.push((*from_line).clone());
                from_lines.next();
                to_lines.next();
                continue;
            }
        }

        // Store context block
        if sync && (!current_block_from.is_empty() || !current_block_to.is_empty()) {
            blocks.push(HunkSegment::Context(current_block_from));
            current_block_from = Vec::new();
            current_block_to = Vec::new();
        }
        sync = false;

        // Stop loop and store last block if no lines left
        if from_line.is_none() && to_line.is_none() {
            if !current_block_from.is_empty() || !current_block_to.is_empty() {
                match sync {
                    true => blocks.push(HunkSegment::Context(current_block_from)),
                    false => blocks.push(HunkSegment::Change {
                        from: current_block_from,
                        to: current_block_to,
                    }),
                }
            }
            break;
        }

        // Add from line to change block if it is a change
        if let Some(from_line) = from_line
            && !matches!(from_line.indicator, LineValueIndicator::Unchanged)
        {
            current_block_from.push((*from_line).clone());
            from_lines.next();
        }

        // Add to line to change block if it is a change
        if let Some(to_line) = to_line
            && !matches!(to_line.indicator, LineValueIndicator::Unchanged)
        {
            current_block_to.push((*to_line).clone());
            to_lines.next();
        }
    }

    blocks
}
