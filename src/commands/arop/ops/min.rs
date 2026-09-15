//! `MIN` for `AROP`

use super::Operation;
use valkey_module::{ValkeyString, ValkeyValue};

pub struct MinOperation {
    minimum: f64,
    found_elements: bool,
}

impl Default for MinOperation {
    fn default() -> Self {
        Self {
            minimum: f64::MAX,
            found_elements: false,
        }
    }
}

impl Operation for MinOperation {
    fn new() -> Self {
        Self::default()
    }

    fn accumulate(&mut self, value: ValkeyString) {
        if let Ok(float) = value.parse_float() {
            self.minimum = self.minimum.min(float);
            self.found_elements = true;
        }
    }

    fn build_result(self) -> ValkeyValue {
        if self.found_elements {
            self.minimum.into()
        } else {
            ValkeyValue::Null
        }
    }
}

#[cfg(test)]
mod test {
    use super::MinOperation;
    use crate::commands::arop::ops::Operation;
    use crate::test_utils::vkstr;
    use valkey_module::ValkeyValue;

    #[test]
    fn empty() {
        let op = MinOperation::new();

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Null);
    }

    #[test]
    fn all_numbers() {
        let mut op = MinOperation::new();

        op.accumulate(vkstr("4.2"));
        op.accumulate(vkstr("2.3"));
        op.accumulate(vkstr("471.1"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(2.3));
    }

    #[test]
    fn all_non_numbers() {
        let mut op = MinOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Null);
    }

    #[test]
    fn mixed() {
        let mut op = MinOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("4.2"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("2.3"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(2.3));
    }
}
