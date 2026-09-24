//! Implementation of an iterable range

use std::fmt::{Debug, Formatter};

/// A range of positions
pub struct Range {
    start: u64,
    end: u64,
    inner: Box<dyn Iterator<Item = u64>>,
}

impl Debug for Range {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}, {:?}]", self.start, self.end)
    }
}
impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl Eq for Range {}

impl Range {
    /// Builds a new instance
    pub fn new(start: u64, end: u64) -> Self {
        let inner: Box<dyn Iterator<Item = u64>> = if start <= end {
            Box::new(start..=end)
        } else {
            Box::new((end..=start).rev())
        };
        Self { start, end, inner }
    }
}

impl Iterator for Range {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn increasing() {
        let range = Range::new(40, 42);

        assert_eq!(range.start, 40);
        assert_eq!(range.end, 42);

        assert_eq!(range.collect::<Vec<_>>(), vec![40, 41, 42]);
    }
    #[test]
    fn decreasing() {
        let range = Range::new(42, 40);

        assert_eq!(range.start, 42);
        assert_eq!(range.end, 40);

        assert_eq!(range.collect::<Vec<_>>(), vec![42, 41, 40]);
    }
}
