//! Utilities for unit tests

use valkey_module::{ValkeyString, ValkeyValue};

pub fn vkstr<S: Into<String>>(input: S) -> ValkeyString {
    ValkeyString::test(input.into())
}

pub fn u32s_to_vec_value(items: &[i32]) -> Vec<ValkeyValue> {
    items.iter().map(|x| (*x as i64).into()).collect()
}
