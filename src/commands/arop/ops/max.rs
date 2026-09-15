//! `MAX` for `AROP`

use super::Operation;
use valkey_module::{ValkeyString, ValkeyValue};

pub struct MaxOperation {
    maximum: f64,
    found_elements: bool,
}

impl Default for MaxOperation {
    fn default() -> Self {
        Self {
            maximum: f64::MIN,
            found_elements: false,
        }
    }
}

impl Operation for MaxOperation {
    fn new() -> Self {
        Self::default()
    }

    fn accumulate(&mut self, value: ValkeyString) {
        if let Ok(float) = value.parse_float() {
            self.maximum = self.maximum.max(float);
            self.found_elements = true;
        }
    }

    fn build_result(self) -> ValkeyValue {
        if self.found_elements {
            self.maximum.into()
        } else {
            ValkeyValue::Null
        }
    }
}

#[cfg(test)]
mod test {
    use super::MaxOperation;
    use crate::commands::arop::ops::Operation;
    use crate::test_utils::vkstr;
    use valkey_module::ValkeyValue;

    #[test]
    fn empty() {
        let op = MaxOperation::new();

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Null);
    }

    #[test]
    fn all_numbers() {
        let mut op = MaxOperation::new();

        op.accumulate(vkstr("4.2"));
        op.accumulate(vkstr("471.1"));
        op.accumulate(vkstr("2.3"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(471.1));
    }

    #[test]
    fn all_non_numbers() {
        let mut op = MaxOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Null);
    }

    #[test]
    fn mixed() {
        let mut op = MaxOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("4.2"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("2.3"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Float(4.2));
    }
}
