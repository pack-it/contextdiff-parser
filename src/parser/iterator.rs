use std::{iter::Peekable, str::Lines};

/// Iterator to iterate over lines and keep the current line index.
pub struct LineIterator<'a> {
    iterator: Peekable<Lines<'a>>,
    current_index: usize,
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = &'a str;

    /// Advances the iterator and returns the next value.
    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some(val) => {
                self.current_index += 1;
                Some(val)
            },
            None => None,
        }
    }
}

impl<'a> LineIterator<'a> {
    /// Creates a new LineIterator, iterating over the lines in the given string.
    pub fn from_lines(string: &'a str) -> Self {
        Self {
            iterator: string.lines().peekable(),
            current_index: 0,
        }
    }

    /// Returns a reference to the next() value without advancing the iterator.
    pub fn peek(&mut self) -> Option<&&'a str> {
        self.iterator.peek()
    }

    /// Returns the current line index of the iterator.
    pub fn index(&self) -> usize {
        self.current_index
    }
}
