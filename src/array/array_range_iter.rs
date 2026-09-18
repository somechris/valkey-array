use crate::array::ArrayType;
use valkey_module::ValkeyString;

/// Generic iterator over a range of an [`Array`]
pub struct GenericArrayRangeIter<'a, ARR: ArrayType> {
    /// The next position to get
    next: u64,
    /// The last position to get
    end: u64,
    /// The source to retrieve elements from
    source: &'a ARR,
}

impl<'a, ARR: ArrayType> GenericArrayRangeIter<'a, ARR> {
    pub fn new(mut start: u64, mut end: u64, source: &'a ARR) -> Self {
        if end < start {
            std::mem::swap(&mut end, &mut start);
        }
        Self {
            next: start,
            end,
            source,
        }
    }
}

impl<'a, ARR: ArrayType> Iterator for GenericArrayRangeIter<'a, ARR> {
    type Item = (u64, Option<&'a ValkeyString>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.end {
            return None;
        }

        let item = (self.next, self.source.get(self.next));
        self.next += 1;
        Some(item)
    }
}
