//! `OR` for `AROP`

use super::Operation;
use crate::commands::arop::ops::utils::vkstring_to_floored_i64;
use valkey_module::{ValkeyString, ValkeyValue};

#[derive(Default)]
pub struct OrOperation {
    result: i64,
    found_elements: bool,
}

impl Operation for OrOperation {
    fn new() -> Self {
        Self::default()
    }

    fn accumulate(&mut self, value: ValkeyString) {
        if let Ok(integer) = vkstring_to_floored_i64(value) {
            self.result |= integer;
            self.found_elements = true;
        }
    }

    fn build_result(self) -> ValkeyValue {
        if self.found_elements {
            self.result.into()
        } else {
            ValkeyValue::Null
        }
    }
}

#[cfg(test)]
mod test {
    use super::OrOperation;
    use crate::commands::arop::ops::Operation;
    use crate::test_utils::vkstr;
    use valkey_module::ValkeyValue;

    #[test]
    fn empty() {
        let op = OrOperation::new();

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Null);
    }

    #[test]
    fn all_numbers() {
        let mut op = OrOperation::new();

        op.accumulate(vkstr("-8.2"));
        op.accumulate(vkstr("3.3"));
        op.accumulate(vkstr("1"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Integer(-5));
    }

    #[test]
    fn all_non_numbers() {
        let mut op = OrOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Null);
    }

    #[test]
    fn mixed() {
        let mut op = OrOperation::new();

        op.accumulate(vkstr("foo"));
        op.accumulate(vkstr("32.2"));
        op.accumulate(vkstr("bar"));
        op.accumulate(vkstr("16.3"));
        op.accumulate(vkstr("baz"));

        let result = op.build_result();
        assert_eq!(result, ValkeyValue::Integer(48));
    }
}
