//! Implementation of an iterable range
use std::ops::RangeInclusive;

/// A range of positions
#[derive(Debug, PartialEq, Eq)]
pub struct Range {
    inner: RangeInclusive<u64>,
}

impl Range {
    /// Builds a new instance
    pub fn new(start: u64, end: u64) -> Self {
        Self { inner: start..=end }
    }
}

impl Iterator for Range {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
