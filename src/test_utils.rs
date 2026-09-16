//! Utilities for unit tests

use valkey_module::{ValkeyString, ValkeyValue};

pub fn vkstr<S: Into<String>>(input: S) -> ValkeyString {
    ValkeyString::test(input.into())
}

pub fn u32s_to_vec_value(items: &[i32]) -> Vec<ValkeyValue> {
    items.iter().map(|x| (*x as i64).into()).collect()
}

/// Converts a slice of `(i32, &str)`s to a `Vec<ValkeyValue>`
pub fn u32s_with_vals_to_vec_value(items: &[(i32, &'static str)]) -> Vec<ValkeyValue> {
    items
        .iter()
        .map(|(pos, str)| {
            let vkpos = ValkeyValue::from(*pos as i64);
            let vkstr = vkstr(*str).into();
            ValkeyValue::from(vec![vkpos, vkstr])
        })
        .collect()
}
