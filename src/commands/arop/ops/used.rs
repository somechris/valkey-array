//! `USED` for `AROP`

use super::Operation;
use valkey_module::{ValkeyString, ValkeyValue};

#[derive(Default)]
pub struct UsedOperation {
    counter: i64,
}

impl Operation for UsedOperation {
    fn new() -> Self {
        Self::default()
    }
    fn accumulate(&mut self, _value: ValkeyString) {
        self.counter += 1;
    }

    fn build_result(self) -> ValkeyValue {
        self.counter.into()
    }
}

#[cfg(test)]
mod test {
    use super::UsedOperation;
    use crate::commands::arop::ops::Operation;
    use crate::test_utils::vkstr;
    use valkey_module::ValkeyValue;

    #[test]
    fn empty() {
        let op = UsedOperation::new();

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Integer(0));
    }

    #[test]
    fn mixed() {
        let mut op = UsedOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("4.2"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("2.3"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Integer(5));
    }
}
