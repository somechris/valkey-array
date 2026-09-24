use crate::array::{ArrayType, Range};
use valkey_module::ValkeyString;

/// Generic iterator over a range of an [`Array`]
pub struct GenericArrayRangeIter<'a, ARR: ArrayType> {
    /// The position iterator
    range: Range,
    /// The source to retrieve elements from
    source: &'a ARR,
}

impl<'a, ARR: ArrayType> GenericArrayRangeIter<'a, ARR> {
    pub fn new(range: Range, source: &'a ARR) -> Self {
        Self { range, source }
    }
}

impl<'a, ARR: ArrayType> Iterator for GenericArrayRangeIter<'a, ARR> {
    type Item = (u64, Option<&'a ValkeyString>);

    fn next(&mut self) -> Option<Self::Item> {
        let position = self.range.next()?;
        let item = (position, self.source.get(position));
        Some(item)
    }
}
