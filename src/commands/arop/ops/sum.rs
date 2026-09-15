//! `SUM` for `AROP`

use super::Operation;
use valkey_module::{ValkeyString, ValkeyValue};

#[derive(Default)]
pub struct SumOperation {
    sum: f64,
}

impl Operation for SumOperation {
    fn new() -> Self {
        Self::default()
    }

    fn accumulate(&mut self, value: ValkeyString) {
        if let Ok(float) = value.parse_float() {
            self.sum += float;
        }
    }

    fn build_result(self) -> ValkeyValue {
        self.sum.into()
    }
}

#[cfg(test)]
mod test {
    use super::SumOperation;
    use crate::commands::arop::ops::Operation;
    use crate::test_utils::vkstr;
    use valkey_module::ValkeyValue;

    #[test]
    fn empty() {
        let op = SumOperation::new();

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(0.0));
    }

    #[test]
    fn all_numbers() {
        let mut op = SumOperation::new();

        op.accumulate(vkstr("4.2"));
        op.accumulate(vkstr("2.3"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(6.5));
    }

    #[test]
    fn all_non_numbers() {
        let mut op = SumOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(0.0));
    }

    #[test]
    fn mixed() {
        let mut op = SumOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("4.2"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("2.3"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(6.5));
    }
}
