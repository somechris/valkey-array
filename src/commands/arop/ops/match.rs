//! `MATCH` for `AROP`

use super::Operation;
use valkey_module::{ValkeyString, ValkeyValue};

pub struct MatchOperation {
    search_expr: ValkeyString,
    counter: i64,
}

impl MatchOperation {
    pub(crate) fn new(search_expr: ValkeyString) -> Self {
        Self {
            search_expr,
            counter: 0,
        }
    }
}
impl Operation for MatchOperation {
    fn accumulate(&mut self, value: &ValkeyString) {
        if value == &self.search_expr {
            self.counter += 1;
        }
    }

    fn build_result(self) -> ValkeyValue {
        self.counter.into()
    }
}

#[cfg(test)]
mod test {
    use super::MatchOperation;
    use crate::commands::arop::ops::Operation;
    use crate::test_utils::vkstr;
    use valkey_module::ValkeyValue;

    #[test]
    fn empty() {
        let op = MatchOperation::new(vkstr("foo"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Integer(0));
    }

    #[test]
    fn mixed() {
        let mut op = MatchOperation::new(vkstr("foo"));

        op.accumulate(&vkstr("foo")); // matches
        op.accumulate(&vkstr("bar")); // does not match
        op.accumulate(&vkstr("foo")); // matches
        op.accumulate(&vkstr("  foo  ")); // does not match (padded)
        op.accumulate(&vkstr("foo")); // matches

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Integer(3));
    }
}
